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
) -> Vec<String> {
    let mut args = vec![
        "exec".to_string(),
        "-C".to_string(),
        temp_path.to_str().unwrap().to_string(),
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
    args
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
    Ok(CodexOutput { stdout, stderr, elapsed_secs, model_used })
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
