use serde::{Deserialize, Serialize};

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
    #[serde(default)]
    pub body: Option<String>,
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
    pub non_substantive_hunk_ids: Vec<String>,
    pub questions: Vec<String>,
}

/// Wrapper for Codex command results that includes CLI log output.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResponse {
    pub result: AnalysisResult,
    pub codex_log: String,
    #[serde(default)]
    pub from_cache: bool,
}

/// Response for refine_group command.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RefineResponse {
    pub sub_groups: Vec<IntentGroup>,
    pub codex_log: String,
    #[serde(default)]
    pub from_cache: bool,
}

/// Codex output shape for refine (same structure as analysis but only groups).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefineResult {
    pub groups: Vec<IntentGroup>,
}
