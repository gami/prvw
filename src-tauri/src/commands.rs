use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::Write;
use std::process::Command;

// ───── Data types ─────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PrAuthor {
    pub login: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PrListItem {
    pub number: u64,
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub author: Option<PrAuthor>,
    #[serde(default)]
    pub head_ref_name: Option<String>,
    #[serde(default)]
    pub base_ref_name: Option<String>,
    #[serde(default)]
    pub review_decision: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    pub kind: String, // "add", "remove", "context"
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Hunk {
    pub id: String,
    pub file_path: String,
    pub header: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParsedDiff {
    pub hunks: Vec<Hunk>,
    pub raw: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IntentGroup {
    pub id: String,
    pub title: String,
    pub rationale: String,
    pub risk: String,
    pub hunk_ids: Vec<String>,
    pub reviewer_checklist: Vec<String>,
    pub suggested_tests: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResult {
    pub version: u32,
    pub overall_summary: String,
    pub groups: Vec<IntentGroup>,
    pub unassigned_hunk_ids: Vec<String>,
    pub questions: Vec<String>,
}

// ───── Helpers ─────

fn validate_repo(repo: &str) -> Result<(), String> {
    // Must be "owner/repo" format
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2
        || parts[0].is_empty()
        || parts[1].is_empty()
        || parts.iter().any(|p| p.contains(|c: char| c.is_whitespace()))
    {
        return Err(format!(
            "Invalid repo format: '{}'. Expected 'owner/repo'.",
            repo
        ));
    }
    Ok(())
}

fn gh_env() -> Vec<(&'static str, &'static str)> {
    vec![
        ("GH_PAGER", "cat"),
        ("PAGER", "cat"),
        ("NO_COLOR", "1"),
        ("GH_FORCE_TTY", "0"),
    ]
}

// ───── Commands ─────

#[tauri::command]
pub async fn list_prs(
    repo: String,
    limit: u32,
    state: String,
    search: Option<String>,
) -> Result<Vec<PrListItem>, String> {
    validate_repo(&repo)?;

    let mut args = vec![
        "pr".to_string(),
        "list".to_string(),
        "-R".to_string(),
        repo.clone(),
        "--state".to_string(),
        state,
        "--limit".to_string(),
        limit.to_string(),
        "--json".to_string(),
        "number,title,author,updatedAt,url,headRefName,baseRefName,reviewDecision".to_string(),
    ];

    if let Some(s) = search {
        if !s.trim().is_empty() {
            args.push("--search".to_string());
            args.push(s);
        }
    }

    let output = Command::new("gh")
        .args(&args)
        .envs(gh_env())
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "GitHub CLI (gh) is not installed. Please install it: https://cli.github.com/"
                    .to_string()
            } else {
                format!("Failed to execute gh: {}", e)
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("auth login") || stderr.contains("not logged") {
            return Err(
                "GitHub CLI is not authenticated. Please run: gh auth login".to_string()
            );
        }
        return Err(format!("gh pr list failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let items: Vec<PrListItem> =
        serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse gh output: {}", e))?;
    Ok(items)
}

#[tauri::command]
pub async fn get_pr_diff(repo: String, pr_number: u32) -> Result<String, String> {
    validate_repo(&repo)?;

    let output = Command::new("gh")
        .args([
            "pr",
            "diff",
            "-R",
            &repo,
            &pr_number.to_string(),
            "--patch",
            "--color",
            "never",
        ])
        .envs(gh_env())
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "GitHub CLI (gh) is not installed.".to_string()
            } else {
                format!("Failed to execute gh: {}", e)
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gh pr diff failed: {}", stderr));
    }

    let diff = String::from_utf8_lossy(&output.stdout).to_string();
    if diff.trim().is_empty() {
        return Err("Diff is empty. The PR may have no changes.".to_string());
    }
    Ok(diff)
}

#[tauri::command]
pub fn parse_diff(diff_text: String) -> Result<ParsedDiff, String> {
    let hunks = parse_unified_diff(&diff_text)?;
    Ok(ParsedDiff {
        hunks,
        raw: diff_text,
    })
}

fn parse_unified_diff(diff_text: &str) -> Result<Vec<Hunk>, String> {
    let hunk_header_re =
        regex::Regex::new(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@(.*)$").unwrap();

    let mut hunks: Vec<Hunk> = Vec::new();
    let mut current_file: Option<String> = None;
    let mut hunk_counter: u32 = 0;
    let mut current_hunk: Option<HunkBuilder> = None;

    struct HunkBuilder {
        file_path: String,
        header: String,
        old_start: u32,
        old_lines: u32,
        new_start: u32,
        new_lines: u32,
        lines: Vec<DiffLine>,
        old_line: u32,
        new_line: u32,
    }

    for line in diff_text.lines() {
        // Detect file path from diff header
        if line.starts_with("diff --git ") || line.starts_with("diff --combined ") {
            // Flush current hunk
            if let Some(hb) = current_hunk.take() {
                hunk_counter += 1;
                hunks.push(Hunk {
                    id: format!("H{}", hunk_counter),
                    file_path: hb.file_path,
                    header: hb.header,
                    old_start: hb.old_start,
                    old_lines: hb.old_lines,
                    new_start: hb.new_start,
                    new_lines: hb.new_lines,
                    lines: hb.lines,
                });
            }
            current_file = None;
            continue;
        }

        if line.starts_with("+++ b/") {
            current_file = Some(line[6..].to_string());
            continue;
        }
        if line.starts_with("+++ /dev/null") {
            // File was deleted; keep file path from ---
            continue;
        }
        if line.starts_with("--- a/") || line.starts_with("--- /dev/null") {
            if current_file.is_none() && line.starts_with("--- a/") {
                current_file = Some(line[6..].to_string());
            }
            continue;
        }

        // Hunk header
        if let Some(caps) = hunk_header_re.captures(line) {
            // Flush previous hunk
            if let Some(hb) = current_hunk.take() {
                hunk_counter += 1;
                hunks.push(Hunk {
                    id: format!("H{}", hunk_counter),
                    file_path: hb.file_path,
                    header: hb.header,
                    old_start: hb.old_start,
                    old_lines: hb.old_lines,
                    new_start: hb.new_start,
                    new_lines: hb.new_lines,
                    lines: hb.lines,
                });
            }

            let old_start: u32 = caps[1].parse().unwrap_or(0);
            let old_lines: u32 = caps.get(2).map_or(1, |m| m.as_str().parse().unwrap_or(1));
            let new_start: u32 = caps[3].parse().unwrap_or(0);
            let new_lines: u32 = caps.get(4).map_or(1, |m| m.as_str().parse().unwrap_or(1));

            let file_path = current_file.clone().unwrap_or_else(|| "unknown".to_string());

            current_hunk = Some(HunkBuilder {
                file_path,
                header: line.to_string(),
                old_start,
                old_lines,
                new_start,
                new_lines,
                lines: Vec::new(),
                old_line: old_start,
                new_line: new_start,
            });
            continue;
        }

        // Diff content lines
        if let Some(ref mut hb) = current_hunk {
            if line.starts_with('+') {
                hb.lines.push(DiffLine {
                    kind: "add".to_string(),
                    old_line: None,
                    new_line: Some(hb.new_line),
                    text: line[1..].to_string(),
                });
                hb.new_line += 1;
            } else if line.starts_with('-') {
                hb.lines.push(DiffLine {
                    kind: "remove".to_string(),
                    old_line: Some(hb.old_line),
                    new_line: None,
                    text: line[1..].to_string(),
                });
                hb.old_line += 1;
            } else if line.starts_with(' ') || line.is_empty() {
                let text = if line.is_empty() {
                    String::new()
                } else {
                    line[1..].to_string()
                };
                hb.lines.push(DiffLine {
                    kind: "context".to_string(),
                    old_line: Some(hb.old_line),
                    new_line: Some(hb.new_line),
                    text,
                });
                hb.old_line += 1;
                hb.new_line += 1;
            } else if line.starts_with('\\') {
                // "\ No newline at end of file" — skip
                continue;
            }
        }
    }

    // Flush last hunk
    if let Some(hb) = current_hunk.take() {
        hunk_counter += 1;
        hunks.push(Hunk {
            id: format!("H{}", hunk_counter),
            file_path: hb.file_path,
            header: hb.header,
            old_start: hb.old_start,
            old_lines: hb.old_lines,
            new_start: hb.new_start,
            new_lines: hb.new_lines,
            lines: hb.lines,
        });
    }

    Ok(hunks)
}

#[tauri::command]
pub async fn analyze_intents_with_codex(hunks_json: String, model: Option<String>, lang: Option<String>) -> Result<AnalysisResult, String> {
    // Parse input hunks to get hunk ids for validation
    let hunks: Vec<Hunk> =
        serde_json::from_str(&hunks_json).map_err(|e| format!("Invalid hunks JSON: {}", e))?;
    let valid_ids: HashSet<String> = hunks.iter().map(|h| h.id.clone()).collect();

    if valid_ids.is_empty() {
        return Err("No hunks to analyze.".to_string());
    }

    // Create temp dir
    let temp_dir =
        tempfile::tempdir().map_err(|e| format!("Failed to create temp directory: {}", e))?;
    let temp_path = temp_dir.path();

    // Write hunks.json
    let hunks_path = temp_path.join("hunks.json");
    let mut hunks_file =
        std::fs::File::create(&hunks_path).map_err(|e| format!("Failed to create hunks.json: {}", e))?;
    hunks_file
        .write_all(hunks_json.as_bytes())
        .map_err(|e| format!("Failed to write hunks.json: {}", e))?;

    // Write schema.json
    let schema = r#"{
  "type":"object",
  "properties":{
    "version":{"type":"integer","enum":[1]},
    "overallSummary":{"type":"string"},
    "groups":{
      "type":"array",
      "items":{
        "type":"object",
        "properties":{
          "id":{"type":"string"},
          "title":{"type":"string"},
          "rationale":{"type":"string"},
          "risk":{"type":"string","enum":["low","medium","high"]},
          "hunkIds":{"type":"array","items":{"type":"string"}},
          "reviewerChecklist":{"type":"array","items":{"type":"string"}},
          "suggestedTests":{"type":"array","items":{"type":"string"}}
        },
        "required":["id","title","rationale","risk","hunkIds","reviewerChecklist","suggestedTests"],
        "additionalProperties":false
      }
    },
    "unassignedHunkIds":{"type":"array","items":{"type":"string"}},
    "questions":{"type":"array","items":{"type":"string"}}
  },
  "required":["version","overallSummary","groups","unassignedHunkIds","questions"],
  "additionalProperties":false
}"#;
    let schema_path = temp_path.join("schema.json");
    let mut schema_file =
        std::fs::File::create(&schema_path).map_err(|e| format!("Failed to create schema.json: {}", e))?;
    schema_file
        .write_all(schema.as_bytes())
        .map_err(|e| format!("Failed to write schema.json: {}", e))?;

    // Run codex exec
    let schema_abs = schema_path.to_str().unwrap();
    let analysis_abs = temp_path.join("analysis.json");
    let analysis_abs_str = analysis_abs.to_str().unwrap();

    let lang_instruction = match lang.as_deref() {
        Some(l) if !l.trim().is_empty() => format!(" Respond in {}.", l.trim()),
        _ => String::new(),
    };
    let prompt = format!(
        "Read hunks.json and group hunks by change intent for PR review. \
         Use only existing hunk ids. Output must match the schema. Do not invent ids. \
         Order the groups array by logical processing flow \
         (e.g. data model / schema first, then business logic, then API / controller, then UI, then tests, then config). \
         Give each group a clear, descriptive title that serves as a section heading for reviewers.{}",
        lang_instruction
    );

    let args = build_codex_args(
        temp_path,
        schema_abs,
        analysis_abs_str,
        &model,
        prompt,
    );

    run_codex(&args)?;

    // Read analysis.json
    let analysis_path = temp_path.join("analysis.json");
    let analysis_str = std::fs::read_to_string(&analysis_path)
        .map_err(|e| format!("Failed to read analysis.json: {}. Codex may not have produced output.", e))?;

    let result: AnalysisResult = serde_json::from_str(&analysis_str)
        .map_err(|e| format!("Failed to parse analysis.json: {}", e))?;

    // Validate result
    validate_analysis(&result, &valid_ids)?;

    Ok(result)
}

// ───── Split large hunks ─────

const HUNK_LINE_THRESHOLD: usize = 100;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SplitResult {
    splits: Vec<SplitEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SplitEntry {
    original_hunk_id: String,
    sub_hunks: Vec<SubHunkRange>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubHunkRange {
    id: String,
    title: String,
    start_line_index: usize,
    end_line_index: usize, // exclusive
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

fn run_codex(args: &[String]) -> Result<(), String> {
    let output = Command::new("codex")
        .args(args)
        .envs(gh_env())
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "Codex CLI is not installed. Please install it: https://github.com/openai/codex"
                    .to_string()
            } else {
                format!("Failed to execute codex: {}", e)
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("login") || stderr.contains("auth") || stderr.contains("API key") {
            return Err("Codex CLI is not authenticated. Please run: codex login".to_string());
        }
        return Err(format!("Codex exec failed: {}", stderr));
    }
    Ok(())
}

#[tauri::command]
pub async fn split_large_hunks(
    hunks_json: String,
    model: Option<String>,
    lang: Option<String>,
) -> Result<Vec<Hunk>, String> {
    let hunks: Vec<Hunk> =
        serde_json::from_str(&hunks_json).map_err(|e| format!("Invalid hunks JSON: {}", e))?;

    let large_hunks: Vec<&Hunk> = hunks
        .iter()
        .filter(|h| h.lines.len() > HUNK_LINE_THRESHOLD)
        .collect();

    if large_hunks.is_empty() {
        return Ok(hunks);
    }

    // Prepare temp dir
    let temp_dir =
        tempfile::tempdir().map_err(|e| format!("Failed to create temp directory: {}", e))?;
    let temp_path = temp_dir.path();

    // Write only large hunks
    let large_json = serde_json::to_string(&large_hunks)
        .map_err(|e| format!("Failed to serialize large hunks: {}", e))?;
    let hunks_path = temp_path.join("large_hunks.json");
    std::fs::write(&hunks_path, &large_json)
        .map_err(|e| format!("Failed to write large_hunks.json: {}", e))?;

    // Write split schema
    let schema = r#"{
  "type":"object",
  "properties":{
    "splits":{
      "type":"array",
      "items":{
        "type":"object",
        "properties":{
          "originalHunkId":{"type":"string"},
          "subHunks":{
            "type":"array",
            "items":{
              "type":"object",
              "properties":{
                "id":{"type":"string"},
                "title":{"type":"string"},
                "startLineIndex":{"type":"integer"},
                "endLineIndex":{"type":"integer"}
              },
              "required":["id","title","startLineIndex","endLineIndex"],
              "additionalProperties":false
            }
          }
        },
        "required":["originalHunkId","subHunks"],
        "additionalProperties":false
      }
    }
  },
  "required":["splits"],
  "additionalProperties":false
}"#;
    let schema_path = temp_path.join("split_schema.json");
    std::fs::write(&schema_path, schema)
        .map_err(|e| format!("Failed to write split_schema.json: {}", e))?;

    let output_path = temp_path.join("split_result.json");

    let lang_instruction = match lang.as_deref() {
        Some(l) if !l.trim().is_empty() => format!(" Respond in {}.", l.trim()),
        _ => String::new(),
    };

    let prompt = format!(
        "Read large_hunks.json. Each hunk has an id, filePath, and a lines array. \
         For each hunk, split it into semantic sub-hunks by change purpose. \
         Each sub-hunk must be a contiguous range of lines (0-based indices, endLineIndex is exclusive). \
         Sub-hunk ids must be \"<originalId>.1\", \"<originalId>.2\", etc. \
         The sub-hunks must cover all lines of the original hunk with no gaps or overlaps. \
         Give each sub-hunk a short descriptive title. \
         Output must match the schema.{}",
        lang_instruction
    );

    let args = build_codex_args(
        temp_path,
        schema_path.to_str().unwrap(),
        output_path.to_str().unwrap(),
        &model,
        prompt,
    );

    run_codex(&args)?;

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
            // Find the split entry
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

                // Compute oldStart/newStart from the first line
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

    Ok(result_hunks)
}

fn validate_analysis(result: &AnalysisResult, valid_ids: &HashSet<String>) -> Result<(), String> {
    let mut all_referenced: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    // Check group hunk ids
    for group in &result.groups {
        for hid in &group.hunk_ids {
            if !valid_ids.contains(hid) {
                return Err(format!(
                    "Analysis references non-existent hunk id: '{}' in group '{}'",
                    hid, group.title
                ));
            }
            if seen.contains(hid) {
                return Err(format!(
                    "Duplicate hunk assignment: '{}' appears in multiple groups",
                    hid
                ));
            }
            seen.insert(hid.clone());
            all_referenced.push(hid.clone());
        }
    }

    // Check unassigned hunk ids
    for hid in &result.unassigned_hunk_ids {
        if !valid_ids.contains(hid) {
            return Err(format!(
                "Analysis references non-existent unassigned hunk id: '{}'",
                hid
            ));
        }
        if seen.contains(hid) {
            return Err(format!(
                "Duplicate hunk assignment: '{}' appears in both groups and unassigned",
                hid
            ));
        }
        seen.insert(hid.clone());
        all_referenced.push(hid.clone());
    }

    // Check coverage
    let missing: Vec<&String> = valid_ids.iter().filter(|id| !seen.contains(*id)).collect();
    if !missing.is_empty() {
        return Err(format!(
            "Analysis is missing hunk ids: {:?}. All hunks must be in groups or unassignedHunkIds.",
            missing
        ));
    }

    Ok(())
}
