use std::collections::HashSet;

use crate::types::AnalysisResult;

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

    ValidationResult { cleaned, warnings }
}
