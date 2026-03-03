use std::process::Command;
use std::time::Instant;

fn codex_env() -> Vec<(&'static str, &'static str)> {
    vec![
        ("GH_PAGER", "cat"),
        ("PAGER", "cat"),
        ("NO_COLOR", "1"),
        ("GH_FORCE_TTY", "0"),
    ]
}

pub fn lang_suffix(lang: &Option<String>) -> String {
    match lang.as_deref() {
        Some(l) if !l.trim().is_empty() => format!(" Respond in {}.", l.trim()),
        _ => String::new(),
    }
}

pub struct CodexOutput {
    pub stdout: String,
    pub stderr: String,
    pub elapsed_secs: f64,
    pub model_used: String,
}

/// Build CLI arguments for Codex exec, write input files, and return args vector.
pub fn build_args(
    temp_path: &std::path::Path,
    schema_path: &str,
    output_path: &str,
    model: &Option<String>,
    prompt: String,
) -> Result<Vec<String>, String> {
    let mut args = vec![
        "exec".to_string(),
        "-C".to_string(),
        temp_path
            .to_str()
            .ok_or_else(|| "Non-UTF-8 temp path".to_string())?
            .to_string(),
        "--skip-git-repo-check".to_string(),
        "--full-auto".to_string(),
        "--sandbox".to_string(),
        "read-only".to_string(),
        "--color".to_string(),
        "never".to_string(),
        "--output-schema".to_string(),
        schema_path.to_string(),
        "-o".to_string(),
        output_path.to_string(),
    ];

    if let Some(m) = model {
        if !m.trim().is_empty() {
            args.push("-m".to_string());
            args.push(m.trim().to_string());
        }
    }

    args.push(prompt);
    Ok(args)
}

/// Run Codex CLI with the given args and return captured output.
pub fn run(args: &[String]) -> Result<CodexOutput, String> {
    let model_used = args
        .windows(2)
        .find(|w| w[0] == "-m")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| "(config default)".to_string());

    let start = Instant::now();
    let output = Command::new("codex")
        .args(args)
        .envs(codex_env())
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "Codex CLI is not installed. Please install it: https://github.com/openai/codex"
                    .to_string()
            } else {
                format!("Failed to execute codex: {}", e)
            }
        })?;
    let elapsed_secs = start.elapsed().as_secs_f64();

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        if stderr.contains("login") || stderr.contains("auth") || stderr.contains("API key") {
            return Err("Codex CLI is not authenticated. Please run: codex login".to_string());
        }
        return Err(format!("Codex exec failed: {}", stderr));
    }
    Ok(CodexOutput {
        stdout,
        stderr,
        elapsed_secs,
        model_used,
    })
}

/// Build a structured log string from Codex output.
pub fn build_log(label: &str, output: &CodexOutput) -> String {
    let mut log = format!(
        "[{}] model={} elapsed={:.1}s\n",
        label, output.model_used, output.elapsed_secs
    );
    if !output.stderr.is_empty() {
        log.push_str(&output.stderr);
        log.push('\n');
    }
    if !output.stdout.is_empty() {
        log.push_str(&output.stdout);
        log.push('\n');
    }
    log
}

/// Prepare a temp directory with hunks.json and schema.json, returning
/// (temp_dir, schema_path, output_path) for the caller to use.
pub fn prepare_temp_dir(
    hunks_json: &str,
    schema_content: &str,
    output_filename: &str,
) -> Result<(tempfile::TempDir, std::path::PathBuf, std::path::PathBuf), String> {
    let temp_dir =
        tempfile::tempdir().map_err(|e| format!("Failed to create temp directory: {}", e))?;
    let temp_path = temp_dir.path();

    std::fs::write(temp_path.join("hunks.json"), hunks_json)
        .map_err(|e| format!("Failed to write hunks.json: {}", e))?;

    let schema_path = temp_path.join("schema.json");
    std::fs::write(&schema_path, schema_content)
        .map_err(|e| format!("Failed to write schema.json: {}", e))?;

    let output_path = temp_path.join(output_filename);

    Ok((temp_dir, schema_path, output_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lang_suffix_none() {
        assert_eq!(lang_suffix(&None), "");
    }

    #[test]
    fn lang_suffix_empty_string() {
        assert_eq!(lang_suffix(&Some(String::new())), "");
    }

    #[test]
    fn lang_suffix_whitespace_only() {
        assert_eq!(lang_suffix(&Some("   ".to_string())), "");
    }

    #[test]
    fn lang_suffix_japanese() {
        assert_eq!(
            lang_suffix(&Some("Japanese".to_string())),
            " Respond in Japanese."
        );
    }

    #[test]
    fn lang_suffix_trims_whitespace() {
        assert_eq!(
            lang_suffix(&Some("  English  ".to_string())),
            " Respond in English."
        );
    }

    #[test]
    fn build_args_without_model() {
        let tmp = tempfile::tempdir().unwrap();
        let args = build_args(
            tmp.path(),
            "/schema.json",
            "/output.json",
            &None,
            "prompt text".to_string(),
        )
        .unwrap();
        assert!(args.contains(&"exec".to_string()));
        assert!(args.contains(&"--full-auto".to_string()));
        assert!(args.contains(&"--sandbox".to_string()));
        assert!(args.contains(&"read-only".to_string()));
        assert!(args.contains(&"--output-schema".to_string()));
        assert!(!args.contains(&"-m".to_string()));
        // Last arg is the prompt
        assert_eq!(args.last().unwrap(), "prompt text");
    }

    #[test]
    fn build_args_with_model() {
        let tmp = tempfile::tempdir().unwrap();
        let args = build_args(
            tmp.path(),
            "/schema.json",
            "/output.json",
            &Some("gpt-4".to_string()),
            "prompt".to_string(),
        )
        .unwrap();
        let m_pos = args.iter().position(|a| a == "-m").unwrap();
        assert_eq!(args[m_pos + 1], "gpt-4");
    }

    #[test]
    fn build_args_empty_model_ignored() {
        let tmp = tempfile::tempdir().unwrap();
        let args = build_args(
            tmp.path(),
            "/schema.json",
            "/output.json",
            &Some("  ".to_string()),
            "prompt".to_string(),
        )
        .unwrap();
        assert!(!args.contains(&"-m".to_string()));
    }

    #[test]
    fn build_log_with_stderr_and_stdout() {
        let output = CodexOutput {
            stdout: "stdout text".to_string(),
            stderr: "stderr text".to_string(),
            elapsed_secs: 1.5,
            model_used: "gpt-4".to_string(),
        };
        let log = build_log("test", &output);
        assert!(log.contains("[test]"));
        assert!(log.contains("model=gpt-4"));
        assert!(log.contains("1.5s"));
        assert!(log.contains("stderr text"));
        assert!(log.contains("stdout text"));
    }

    #[test]
    fn build_log_empty_stderr_omitted() {
        let output = CodexOutput {
            stdout: "out".to_string(),
            stderr: String::new(),
            elapsed_secs: 0.0,
            model_used: "m".to_string(),
        };
        let log = build_log("x", &output);
        // Should have header + stdout, no extra empty stderr section
        let lines: Vec<&str> = log.lines().collect();
        assert_eq!(lines[0], "[x] model=m elapsed=0.0s");
        assert_eq!(lines[1], "out");
    }

    #[test]
    fn prepare_temp_dir_creates_files() {
        let (temp_dir, schema_path, output_path) =
            prepare_temp_dir("{}", "{\"type\":\"object\"}", "out.json").unwrap();
        let temp_path = temp_dir.path();
        assert!(temp_path.join("hunks.json").exists());
        assert!(schema_path.exists());
        // output_path should not exist yet (codex writes it)
        assert!(!output_path.exists());
        let hunks = std::fs::read_to_string(temp_path.join("hunks.json")).unwrap();
        assert_eq!(hunks, "{}");
    }
}
