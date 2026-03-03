use std::collections::HashSet;

use crate::cache;
use crate::codex_runner::{self, lang_suffix};
use crate::types::{AnalysisResponse, AnalysisResult, Hunk, RefineResponse, RefineResult};
use crate::validation::validate_analysis;

const ANALYSIS_SCHEMA: &str = include_str!("../schemas/analysis.json");
const REFINE_SCHEMA: &str = include_str!("../schemas/refine.json");

fn build_analysis_prompt(
    hunk_count: usize,
    pr_body: &Option<String>,
    lang: &Option<String>,
) -> String {
    let pr_context = match pr_body.as_deref() {
        Some(body) if !body.trim().is_empty() => {
            let truncated = if body.len() > 2000 {
                let end = body.floor_char_boundary(2000);
                &body[..end]
            } else {
                body
            };
            format!(" The PR description is: \"{}\".", truncated)
        }
        _ => String::new(),
    };

    format!(
        "Read hunks.json which contains {} hunks and group ALL of them by change intent for PR review.{} \
         Every single hunk must be assigned to exactly one group — do not leave any hunk unassigned. \
         Use only existing hunk ids. Output must match the schema. Do not invent ids. \
         Order the groups array by logical processing flow \
         (e.g. data model / schema first, then business logic, then API / controller, then UI, then tests, then config). \
         Give each group a clear, descriptive title that serves as a section heading for reviewers. \
         Assign each group a category from: schema, logic, api, ui, test, config, docs, refactor, other. \
         For overallSummary, write a concise reviewer-facing summary of WHAT the PR changes and WHY. \
         Do NOT mention hunks, hunks.json, grouping process, or analysis internals — write as if summarizing the PR itself. \
         Also classify each hunk as substantive or non-substantive. \
         Non-substantive changes are: formatting/whitespace-only changes, code moved to another file without modification, \
         indentation changes, lock file updates, auto-generated code changes, snapshot updates. \
         Note: variable/function renames and comment changes ARE substantive. \
         List non-substantive hunk IDs in nonSubstantiveHunkIds.{}",
        hunk_count,
        pr_context,
        lang_suffix(lang)
    )
}

fn build_refine_prompt(group_title: &str, group_id: &str, lang: &Option<String>) -> String {
    format!(
        "Read hunks.json. These hunks all belong to a single intent group titled \"{}\". \
         Split them into smaller, more focused sub-groups by specific change purpose. \
         Use only existing hunk ids from the input. Do not invent ids. \
         Sub-group ids must be \"{}.1\", \"{}.2\", etc. \
         Order sub-groups by logical processing flow. \
         Give each sub-group a clear, descriptive title. \
         Assign each sub-group a category from: schema, logic, api, ui, test, config, docs, refactor, other.{}",
        group_title, group_id, group_id, lang_suffix(lang)
    )
}

#[tauri::command]
pub async fn analyze_intents_with_codex(
    app: tauri::AppHandle,
    hunks_json: String,
    pr_body: Option<String>,
    model: Option<String>,
    lang: Option<String>,
    force: Option<bool>,
) -> Result<AnalysisResponse, String> {
    use tauri::Manager;

    let hunks: Vec<Hunk> =
        serde_json::from_str(&hunks_json).map_err(|e| format!("Invalid hunks JSON: {}", e))?;
    let valid_ids: HashSet<String> = hunks.iter().map(|h| h.id.clone()).collect();

    if valid_ids.is_empty() {
        return Err("No hunks to analyze.".to_string());
    }

    let app_data_dir = app.path().app_data_dir().ok();
    let model_str = model.as_deref().unwrap_or("");
    let lang_str = lang.as_deref().unwrap_or("");
    let pr_body_str = pr_body.as_deref().unwrap_or("");
    let cache_key = cache::hash_key(&format!(
        "{}\n{}\n{}\n{}",
        hunks_json, pr_body_str, model_str, lang_str
    ));

    // Check cache (unless force)
    if force != Some(true) {
        if let Some(ref dir) = app_data_dir {
            if let Some(mut cached) =
                cache::read_cache::<AnalysisResponse>(dir, "cache/analysis", &cache_key)
            {
                cached.from_cache = true;
                return Ok(cached);
            }
        }
    }

    let (temp_dir, schema_path, output_path) =
        codex_runner::prepare_temp_dir(&hunks_json, ANALYSIS_SCHEMA, "analysis.json")?;

    let prompt = build_analysis_prompt(valid_ids.len(), &pr_body, &lang);

    let args = codex_runner::build_args(
        temp_dir.path(),
        schema_path
            .to_str()
            .ok_or_else(|| "Non-UTF-8 schema path".to_string())?,
        output_path
            .to_str()
            .ok_or_else(|| "Non-UTF-8 output path".to_string())?,
        &model,
        prompt,
    )?;

    let codex_output = codex_runner::run(&args)?;

    let analysis_str = std::fs::read_to_string(&output_path).map_err(|e| {
        format!(
            "Failed to read analysis.json: {}. Codex may not have produced output.",
            e
        )
    })?;

    let result: AnalysisResult = serde_json::from_str(&analysis_str)
        .map_err(|e| format!("Failed to parse analysis.json: {}", e))?;

    let validation = validate_analysis(&result, &valid_ids);

    let mut log = codex_runner::build_log("analysis", &codex_output);
    log.push_str(&format!(
        "[analysis] hunks={} groups={}\n",
        valid_ids.len(),
        validation.cleaned.groups.len()
    ));
    if !validation.warnings.is_empty() {
        log.push_str("--- validation warnings ---\n");
        for w in &validation.warnings {
            log.push_str(w);
            log.push('\n');
        }
    }

    let response = AnalysisResponse {
        result: validation.cleaned,
        codex_log: log,
        from_cache: false,
    };

    // Write cache
    if let Some(ref dir) = app_data_dir {
        cache::write_cache(dir, "cache/analysis", &cache_key, &response);
    }

    Ok(response)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn refine_group(
    app: tauri::AppHandle,
    hunks_json: String,
    group_id: String,
    group_title: String,
    hunk_ids: Vec<String>,
    model: Option<String>,
    lang: Option<String>,
    force: Option<bool>,
) -> Result<RefineResponse, String> {
    use tauri::Manager;

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

    let group_hunks_json = serde_json::to_string(&group_hunks)
        .map_err(|e| format!("Failed to serialize group hunks: {}", e))?;

    let app_data_dir = app.path().app_data_dir().ok();
    let model_str = model.as_deref().unwrap_or("");
    let lang_str = lang.as_deref().unwrap_or("");
    let cache_key = cache::hash_key(&format!(
        "{}\n{}\n{}\n{}\n{}",
        group_hunks_json, group_id, group_title, model_str, lang_str
    ));

    // Check cache (unless force)
    if force != Some(true) {
        if let Some(ref dir) = app_data_dir {
            if let Some(mut cached) =
                cache::read_cache::<RefineResponse>(dir, "cache/refine", &cache_key)
            {
                cached.from_cache = true;
                return Ok(cached);
            }
        }
    }

    let (temp_dir, schema_path, output_path) =
        codex_runner::prepare_temp_dir(&group_hunks_json, REFINE_SCHEMA, "refine.json")?;

    let prompt = build_refine_prompt(&group_title, &group_id, &lang);

    let args = codex_runner::build_args(
        temp_dir.path(),
        schema_path
            .to_str()
            .ok_or_else(|| "Non-UTF-8 schema path".to_string())?,
        output_path
            .to_str()
            .ok_or_else(|| "Non-UTF-8 output path".to_string())?,
        &model,
        prompt,
    )?;

    let codex_output = codex_runner::run(&args)?;

    let result_str = std::fs::read_to_string(&output_path).map_err(|e| {
        format!(
            "Failed to read refine.json: {}. Codex may not have produced output.",
            e
        )
    })?;

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
                warnings.push(format!(
                    "Removed non-existent hunk id '{}' from sub-group '{}'",
                    id, g.title
                ));
                false
            }
        });
        if g.hunk_ids.len() != before {
            warnings.push(format!(
                "Sub-group '{}': {} -> {} hunks",
                g.title,
                before,
                g.hunk_ids.len()
            ));
        }
    }
    cleaned_groups.retain(|g| !g.hunk_ids.is_empty());

    let mut log = codex_runner::build_log("refine", &codex_output);
    log.push_str(&format!(
        "[refine] group=\"{}\" sub-groups={}\n",
        group_title,
        cleaned_groups.len()
    ));
    if !warnings.is_empty() {
        log.push_str("--- validation warnings ---\n");
        for w in &warnings {
            log.push_str(w);
            log.push('\n');
        }
    }

    let response = RefineResponse {
        sub_groups: cleaned_groups,
        codex_log: log,
        from_cache: false,
    };

    // Write cache
    if let Some(ref dir) = app_data_dir {
        cache::write_cache(dir, "cache/refine", &cache_key, &response);
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_prompt_includes_hunk_count() {
        let prompt = build_analysis_prompt(5, &None, &None);
        assert!(prompt.contains("5 hunks"));
    }

    #[test]
    fn analysis_prompt_no_pr_body() {
        let prompt = build_analysis_prompt(1, &None, &None);
        assert!(!prompt.contains("PR description"));
    }

    #[test]
    fn analysis_prompt_with_pr_body() {
        let body = Some("Fix login bug".to_string());
        let prompt = build_analysis_prompt(1, &body, &None);
        assert!(prompt.contains("Fix login bug"));
        assert!(prompt.contains("PR description"));
    }

    #[test]
    fn analysis_prompt_truncates_long_body() {
        let body = Some("x".repeat(3000));
        let prompt = build_analysis_prompt(1, &body, &None);
        // The body in the prompt should be truncated to ~2000 chars
        assert!(prompt.len() < 3000 + 500);
        assert!(prompt.contains("PR description"));
    }

    #[test]
    fn analysis_prompt_with_lang() {
        let prompt = build_analysis_prompt(1, &None, &Some("Japanese".to_string()));
        assert!(prompt.contains("Respond in Japanese."));
    }

    #[test]
    fn refine_prompt_contains_group_info() {
        let prompt = build_refine_prompt("Auth changes", "G1", &None);
        assert!(prompt.contains("Auth changes"));
        assert!(prompt.contains("G1.1"));
        assert!(prompt.contains("G1.2"));
    }

    #[test]
    fn refine_prompt_with_lang() {
        let prompt = build_refine_prompt("Title", "G1", &Some("Spanish".to_string()));
        assert!(prompt.contains("Respond in Spanish."));
    }
}
