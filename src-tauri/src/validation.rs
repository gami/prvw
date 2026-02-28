use std::collections::HashSet;

use crate::types::AnalysisResult;

/// Validate that a repo string is in "owner/repo" format.
pub fn validate_repo(repo: &str) -> Result<(), String> {
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

pub struct ValidationResult {
    pub cleaned: AnalysisResult,
    pub warnings: Vec<String>,
}

/// Validate and clean up analysis results.
/// Instead of failing on invalid IDs, remove them and collect warnings.
pub fn validate_analysis(
    result: &AnalysisResult,
    valid_ids: &HashSet<String>,
) -> ValidationResult {
    let mut warnings: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut cleaned = result.clone();

    // Clean groups: remove invalid/duplicate hunk IDs
    for group in &mut cleaned.groups {
        let original_len = group.hunk_ids.len();
        group.hunk_ids.retain(|hid| {
            if !valid_ids.contains(hid) {
                warnings.push(format!(
                    "Removed non-existent hunk id '{}' from group '{}'",
                    hid, group.title
                ));
                return false;
            }
            if seen.contains(hid) {
                warnings.push(format!(
                    "Removed duplicate hunk id '{}' in group '{}'",
                    hid, group.title
                ));
                return false;
            }
            seen.insert(hid.clone());
            true
        });
        if group.hunk_ids.len() != original_len {
            warnings.push(format!(
                "Group '{}': {} -> {} hunks after cleanup",
                group.title, original_len, group.hunk_ids.len()
            ));
        }
    }

    // Remove empty groups
    let before = cleaned.groups.len();
    cleaned.groups.retain(|g| !g.hunk_ids.is_empty());
    if cleaned.groups.len() != before {
        warnings.push(format!(
            "Removed {} empty group(s) after cleanup",
            before - cleaned.groups.len()
        ));
    }

    // Clean unassigned: remove invalid/duplicate
    cleaned.unassigned_hunk_ids.retain(|hid| {
        if !valid_ids.contains(hid) {
            warnings.push(format!("Removed non-existent unassigned hunk id '{}'", hid));
            return false;
        }
        if seen.contains(hid) {
            warnings.push(format!(
                "Removed duplicate unassigned hunk id '{}'",
                hid
            ));
            return false;
        }
        seen.insert(hid.clone());
        true
    });

    // Add missing hunks to unassigned
    let missing: Vec<String> = valid_ids
        .iter()
        .filter(|id| !seen.contains(*id))
        .cloned()
        .collect();
    if !missing.is_empty() {
        warnings.push(format!(
            "Added {} missing hunk(s) to unassigned: {:?}",
            missing.len(),
            missing
        ));
        cleaned.unassigned_hunk_ids.extend(missing);
    }

    // Clean nonSubstantiveHunkIds: remove invalid IDs
    let original_ns_len = cleaned.non_substantive_hunk_ids.len();
    cleaned.non_substantive_hunk_ids.retain(|hid| {
        if valid_ids.contains(hid) {
            true
        } else {
            warnings.push(format!(
                "Removed non-existent non-substantive hunk id '{}'",
                hid
            ));
            false
        }
    });
    if cleaned.non_substantive_hunk_ids.len() != original_ns_len {
        warnings.push(format!(
            "nonSubstantiveHunkIds: {} -> {} after cleanup",
            original_ns_len,
            cleaned.non_substantive_hunk_ids.len()
        ));
    }

    ValidationResult { cleaned, warnings }
}
