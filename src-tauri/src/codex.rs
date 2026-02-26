use std::collections::HashSet;
use std::process::Command;
use std::time::Instant;

use crate::types::{AnalysisResponse, AnalysisResult, Hunk, RefineResponse, RefineResult};
use crate::validation::validate_analysis;

const ANALYSIS_SCHEMA: &str = include_str!("../schemas/analysis.json");
const REFINE_SCHEMA: &str = include_str!("../schemas/refine.json");

fn codex_env() -> Vec<(&'static str, &'static str)> {
    vec![
        ("GH_PAGER", "cat"),
        ("PAGER", "cat"),
        ("NO_COLOR", "1"),
        ("GH_FORCE_TTY", "0"),
    ]
}

fn lang_suffix(lang: &Option<String>) -> String {
    match lang.as_deref() {
        Some(l) if !l.trim().is_empty() => format!(" Respond in {}.", l.trim()),
        _ => String::new(),
    }
}

fn build_codex_args(
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

struct CodexOutput {
    stdout: String,
    stderr: String,
    elapsed_secs: f64,
    model_used: String,
}

/// Runs Codex CLI and returns stdout, stderr, elapsed time, and model info.
fn run_codex(args: &[String]) -> Result<CodexOutput, String> {
    // Extract model from args for logging
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

fn build_log(label: &str, output: &CodexOutput) -> String {
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

// ───── Commands ─────

#[tauri::command]
pub async fn analyze_intents_with_codex(
    hunks_json: String,
    model: Option<String>,
    lang: Option<String>,
) -> Result<AnalysisResponse, String> {
    let hunks: Vec<Hunk> =
        serde_json::from_str(&hunks_json).map_err(|e| format!("Invalid hunks JSON: {}", e))?;
    let valid_ids: HashSet<String> = hunks.iter().map(|h| h.id.clone()).collect();

    if valid_ids.is_empty() {
        return Err("No hunks to analyze.".to_string());
    }

    let temp_dir =
        tempfile::tempdir().map_err(|e| format!("Failed to create temp directory: {}", e))?;
    let temp_path = temp_dir.path();

    // Write hunks.json
    std::fs::write(temp_path.join("hunks.json"), &hunks_json)
        .map_err(|e| format!("Failed to write hunks.json: {}", e))?;

    // Write schema.json
    let schema_path = temp_path.join("schema.json");
    std::fs::write(&schema_path, ANALYSIS_SCHEMA)
        .map_err(|e| format!("Failed to write schema.json: {}", e))?;

    let analysis_path = temp_path.join("analysis.json");

    let prompt = format!(
        "Read hunks.json and group hunks by change intent for PR review. \
         Use only existing hunk ids. Output must match the schema. Do not invent ids. \
         Order the groups array by logical processing flow \
         (e.g. data model / schema first, then business logic, then API / controller, then UI, then tests, then config). \
         Give each group a clear, descriptive title that serves as a section heading for reviewers.{}",
        lang_suffix(&lang)
    );

    let args = build_codex_args(
        temp_path,
        schema_path.to_str().unwrap(),
        analysis_path.to_str().unwrap(),
        &model,
        prompt,
    );

    let codex_output = run_codex(&args)?;

    let analysis_str = std::fs::read_to_string(&analysis_path)
        .map_err(|e| format!("Failed to read analysis.json: {}. Codex may not have produced output.", e))?;

    let result: AnalysisResult = serde_json::from_str(&analysis_str)
        .map_err(|e| format!("Failed to parse analysis.json: {}", e))?;

    let validation = validate_analysis(&result, &valid_ids);

    let mut log = build_log("analysis", &codex_output);
    log.push_str(&format!("[analysis] hunks={} groups={}\n", valid_ids.len(), validation.cleaned.groups.len()));
    if !validation.warnings.is_empty() {
        log.push_str("--- validation warnings ---\n");
        for w in &validation.warnings {
            log.push_str(w);
            log.push('\n');
        }
    }
    Ok(AnalysisResponse { result: validation.cleaned, codex_log: log })
}

#[tauri::command]
pub async fn refine_group(
    hunks_json: String,
    group_id: String,
    group_title: String,
    hunk_ids: Vec<String>,
    model: Option<String>,
    lang: Option<String>,
) -> Result<RefineResponse, String> {
    let all_hunks: Vec<Hunk> =
        serde_json::from_str(&hunks_json).map_err(|e| format!("Invalid hunks JSON: {}", e))?;

    let hunk_id_set: HashSet<String> = hunk_ids.into_iter().collect();
    let group_hunks: Vec<&Hunk> = all_hunks
        .iter()
        .filter(|h| hunk_id_set.contains(&h.id))
        .collect();

    if group_hunks.is_empty() {
        return Err("No hunks found for this group.".to_string());
    }

    let temp_dir =
        tempfile::tempdir().map_err(|e| format!("Failed to create temp directory: {}", e))?;
    let temp_path = temp_dir.path();

    let group_hunks_json = serde_json::to_string(&group_hunks)
        .map_err(|e| format!("Failed to serialize group hunks: {}", e))?;
    std::fs::write(temp_path.join("hunks.json"), &group_hunks_json)
        .map_err(|e| format!("Failed to write hunks.json: {}", e))?;

    let schema_path = temp_path.join("schema.json");
    std::fs::write(&schema_path, REFINE_SCHEMA)
        .map_err(|e| format!("Failed to write schema.json: {}", e))?;

    let output_path = temp_path.join("refine.json");

    let prompt = format!(
        "Read hunks.json. These hunks all belong to a single intent group titled \"{}\". \
         Split them into smaller, more focused sub-groups by specific change purpose. \
         Use only existing hunk ids from the input. Do not invent ids. \
         Sub-group ids must be \"{}.1\", \"{}.2\", etc. \
         Order sub-groups by logical processing flow. \
         Give each sub-group a clear, descriptive title.{}",
        group_title, group_id, group_id, lang_suffix(&lang)
    );

    let args = build_codex_args(
        temp_path,
        schema_path.to_str().unwrap(),
        output_path.to_str().unwrap(),
        &model,
        prompt,
    );

    let codex_output = run_codex(&args)?;

    let result_str = std::fs::read_to_string(&output_path)
        .map_err(|e| format!("Failed to read refine.json: {}. Codex may not have produced output.", e))?;

    let refine_result: RefineResult = serde_json::from_str(&result_str)
        .map_err(|e| format!("Failed to parse refine.json: {}", e))?;

    // Validate: strip invalid hunk IDs
    let mut warnings: Vec<String> = Vec::new();
    let mut cleaned_groups = refine_result.groups;
    for g in &mut cleaned_groups {
        let before = g.hunk_ids.len();
        g.hunk_ids.retain(|id| {
            if hunk_id_set.contains(id) {
                true
            } else {
                warnings.push(format!("Removed non-existent hunk id '{}' from sub-group '{}'", id, g.title));
                false
            }
        });
        if g.hunk_ids.len() != before {
            warnings.push(format!("Sub-group '{}': {} -> {} hunks", g.title, before, g.hunk_ids.len()));
        }
    }
    cleaned_groups.retain(|g| !g.hunk_ids.is_empty());

    let mut log = build_log("refine", &codex_output);
    log.push_str(&format!("[refine] group=\"{}\" sub-groups={}\n", group_title, cleaned_groups.len()));
    if !warnings.is_empty() {
        log.push_str("--- validation warnings ---\n");
        for w in &warnings {
            log.push_str(w);
            log.push('\n');
        }
    }

    Ok(RefineResponse {
        sub_groups: cleaned_groups,
        codex_log: log,
    })
}
