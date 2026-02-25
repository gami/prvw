use std::collections::HashSet;
use std::process::Command;
use std::time::Instant;

use crate::types::{AnalysisResponse, AnalysisResult, Hunk, SplitResponse, SplitResult};
use crate::validation::validate_analysis;

const HUNK_LINE_THRESHOLD: usize = 100;
const ANALYSIS_SCHEMA: &str = include_str!("../schemas/analysis.json");
const SPLIT_SCHEMA: &str = include_str!("../schemas/split.json");

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
pub async fn split_large_hunks(
    hunks_json: String,
    model: Option<String>,
    lang: Option<String>,
) -> Result<SplitResponse, String> {
    let hunks: Vec<Hunk> =
        serde_json::from_str(&hunks_json).map_err(|e| format!("Invalid hunks JSON: {}", e))?;

    let large_hunks: Vec<&Hunk> = hunks
        .iter()
        .filter(|h| h.lines.len() > HUNK_LINE_THRESHOLD)
        .collect();

    if large_hunks.is_empty() {
        return Ok(SplitResponse {
            hunks,
            codex_log: String::new(),
        });
    }

    let temp_dir =
        tempfile::tempdir().map_err(|e| format!("Failed to create temp directory: {}", e))?;
    let temp_path = temp_dir.path();

    // Write large hunks
    let large_json = serde_json::to_string(&large_hunks)
        .map_err(|e| format!("Failed to serialize large hunks: {}", e))?;
    std::fs::write(temp_path.join("large_hunks.json"), &large_json)
        .map_err(|e| format!("Failed to write large_hunks.json: {}", e))?;

    // Write schema
    let schema_path = temp_path.join("split_schema.json");
    std::fs::write(&schema_path, SPLIT_SCHEMA)
        .map_err(|e| format!("Failed to write split_schema.json: {}", e))?;

    let output_path = temp_path.join("split_result.json");

    let prompt = format!(
        "Read large_hunks.json. Each hunk has an id, filePath, and a lines array. \
         For each hunk, split it into semantic sub-hunks by change purpose. \
         Each sub-hunk must be a contiguous range of lines (0-based indices, endLineIndex is exclusive). \
         Sub-hunk ids must be \"<originalId>.1\", \"<originalId>.2\", etc. \
         The sub-hunks must cover all lines of the original hunk with no gaps or overlaps. \
         Give each sub-hunk a short descriptive title. \
         Output must match the schema.{}",
        lang_suffix(&lang)
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
        .map_err(|e| format!("Failed to read split_result.json: {}", e))?;
    let split_result: SplitResult = serde_json::from_str(&result_str)
        .map_err(|e| format!("Failed to parse split_result.json: {}", e))?;

    // Apply splits
    let split_ids: HashSet<String> = split_result
        .splits
        .iter()
        .map(|s| s.original_hunk_id.clone())
        .collect();

    let mut result_hunks: Vec<Hunk> = Vec::new();

    for hunk in &hunks {
        if split_ids.contains(&hunk.id) {
            let entry = split_result
                .splits
                .iter()
                .find(|s| s.original_hunk_id == hunk.id)
                .unwrap();

            for sub in &entry.sub_hunks {
                let start = sub.start_line_index.min(hunk.lines.len());
                let end = sub.end_line_index.min(hunk.lines.len());
                if start >= end {
                    continue;
                }
                let sub_lines = hunk.lines[start..end].to_vec();

                let old_start = sub_lines
                    .iter()
                    .find_map(|l| l.old_line)
                    .unwrap_or(hunk.old_start);
                let new_start = sub_lines
                    .iter()
                    .find_map(|l| l.new_line)
                    .unwrap_or(hunk.new_start);
                let old_count = sub_lines.iter().filter(|l| l.kind != "add").count() as u32;
                let new_count = sub_lines.iter().filter(|l| l.kind != "remove").count() as u32;

                result_hunks.push(Hunk {
                    id: sub.id.clone(),
                    file_path: hunk.file_path.clone(),
                    header: format!(
                        "@@ -{},{} +{},{} @@ [{}]",
                        old_start, old_count, new_start, new_count, sub.title
                    ),
                    old_start,
                    old_lines: old_count,
                    new_start,
                    new_lines: new_count,
                    lines: sub_lines,
                });
            }
        } else {
            result_hunks.push(hunk.clone());
        }
    }

    let codex_log = build_log("split", &codex_output);
    Ok(SplitResponse {
        hunks: result_hunks,
        codex_log,
    })
}
