use std::collections::HashSet;

use crate::types::AnalysisResult;

pub struct ValidationResult {
    pub cleaned: AnalysisResult,
    pub warnings: Vec<String>,
}

/// Validate and clean up analysis results.
/// Instead of failing on invalid IDs, remove them and collect warnings.
pub fn validate_analysis(result: &AnalysisResult, valid_ids: &HashSet<String>) -> ValidationResult {
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
                group.title,
                original_len,
                group.hunk_ids.len()
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
            warnings.push(format!("Removed duplicate unassigned hunk id '{}'", hid));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AnalysisResult, IntentGroup};

    fn make_group(id: &str, title: &str, hunk_ids: Vec<&str>) -> IntentGroup {
        IntentGroup {
            id: id.to_string(),
            title: title.to_string(),
            category: "logic".to_string(),
            rationale: String::new(),
            risk: "low".to_string(),
            hunk_ids: hunk_ids.into_iter().map(String::from).collect(),
            reviewer_checklist: vec![],
            suggested_tests: vec![],
        }
    }

    fn make_result(
        groups: Vec<IntentGroup>,
        unassigned: Vec<&str>,
        non_sub: Vec<&str>,
    ) -> AnalysisResult {
        AnalysisResult {
            version: 1,
            overall_summary: String::new(),
            groups,
            unassigned_hunk_ids: unassigned.into_iter().map(String::from).collect(),
            non_substantive_hunk_ids: non_sub.into_iter().map(String::from).collect(),
            questions: vec![],
        }
    }

    fn ids(slice: &[&str]) -> HashSet<String> {
        slice.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn full_coverage_no_warnings() {
        let result = make_result(
            vec![make_group("G1", "Group 1", vec!["H1", "H2"])],
            vec![],
            vec![],
        );
        let valid = ids(&["H1", "H2"]);
        let vr = validate_analysis(&result, &valid);
        assert!(vr.warnings.is_empty());
        assert_eq!(vr.cleaned.groups.len(), 1);
        assert_eq!(vr.cleaned.groups[0].hunk_ids, vec!["H1", "H2"]);
    }

    #[test]
    fn removes_nonexistent_ids_from_groups() {
        let result = make_result(
            vec![make_group("G1", "Group 1", vec!["H1", "H99"])],
            vec![],
            vec![],
        );
        let valid = ids(&["H1"]);
        let vr = validate_analysis(&result, &valid);
        assert_eq!(vr.cleaned.groups[0].hunk_ids, vec!["H1"]);
        assert!(vr.warnings.iter().any(|w| w.contains("H99")));
    }

    #[test]
    fn removes_duplicate_ids_across_groups() {
        let result = make_result(
            vec![
                make_group("G1", "First", vec!["H1", "H2"]),
                make_group("G2", "Second", vec!["H2", "H3"]),
            ],
            vec![],
            vec![],
        );
        let valid = ids(&["H1", "H2", "H3"]);
        let vr = validate_analysis(&result, &valid);
        assert_eq!(vr.cleaned.groups[0].hunk_ids, vec!["H1", "H2"]);
        assert_eq!(vr.cleaned.groups[1].hunk_ids, vec!["H3"]);
        assert!(vr.warnings.iter().any(|w| w.contains("duplicate")));
    }

    #[test]
    fn removes_duplicate_ids_within_group() {
        let result = make_result(
            vec![make_group("G1", "Group", vec!["H1", "H1", "H2"])],
            vec![],
            vec![],
        );
        let valid = ids(&["H1", "H2"]);
        let vr = validate_analysis(&result, &valid);
        assert_eq!(vr.cleaned.groups[0].hunk_ids, vec!["H1", "H2"]);
    }

    #[test]
    fn removes_empty_groups_after_cleanup() {
        let result = make_result(
            vec![
                make_group("G1", "Valid", vec!["H1"]),
                make_group("G2", "Invalid", vec!["H99"]),
            ],
            vec![],
            vec![],
        );
        let valid = ids(&["H1"]);
        let vr = validate_analysis(&result, &valid);
        assert_eq!(vr.cleaned.groups.len(), 1);
        assert_eq!(vr.cleaned.groups[0].title, "Valid");
        assert!(vr.warnings.iter().any(|w| w.contains("empty group")));
    }

    #[test]
    fn adds_missing_hunks_to_unassigned() {
        let result = make_result(vec![make_group("G1", "Group", vec!["H1"])], vec![], vec![]);
        let valid = ids(&["H1", "H2", "H3"]);
        let vr = validate_analysis(&result, &valid);
        let unassigned = &vr.cleaned.unassigned_hunk_ids;
        assert!(unassigned.contains(&"H2".to_string()));
        assert!(unassigned.contains(&"H3".to_string()));
        assert!(vr.warnings.iter().any(|w| w.contains("missing")));
    }

    #[test]
    fn removes_invalid_unassigned_ids() {
        let result = make_result(
            vec![make_group("G1", "Group", vec!["H1"])],
            vec!["H99"],
            vec![],
        );
        let valid = ids(&["H1"]);
        let vr = validate_analysis(&result, &valid);
        assert!(vr.cleaned.unassigned_hunk_ids.is_empty());
        assert!(vr
            .warnings
            .iter()
            .any(|w| w.contains("non-existent unassigned")));
    }

    #[test]
    fn removes_duplicate_unassigned_already_in_group() {
        let result = make_result(
            vec![make_group("G1", "Group", vec!["H1"])],
            vec!["H1"],
            vec![],
        );
        let valid = ids(&["H1"]);
        let vr = validate_analysis(&result, &valid);
        assert!(vr.cleaned.unassigned_hunk_ids.is_empty());
        assert!(vr
            .warnings
            .iter()
            .any(|w| w.contains("duplicate unassigned")));
    }

    #[test]
    fn removes_invalid_non_substantive_ids() {
        let result = make_result(
            vec![make_group("G1", "Group", vec!["H1"])],
            vec![],
            vec!["H1", "H99"],
        );
        let valid = ids(&["H1"]);
        let vr = validate_analysis(&result, &valid);
        assert_eq!(vr.cleaned.non_substantive_hunk_ids, vec!["H1"]);
        assert!(vr.warnings.iter().any(|w| w.contains("non-substantive")));
    }

    #[test]
    fn all_unassigned_with_no_groups() {
        let result = make_result(vec![], vec![], vec![]);
        let valid = ids(&["H1", "H2"]);
        let vr = validate_analysis(&result, &valid);
        assert!(vr.cleaned.groups.is_empty());
        let unassigned = &vr.cleaned.unassigned_hunk_ids;
        assert!(unassigned.contains(&"H1".to_string()));
        assert!(unassigned.contains(&"H2".to_string()));
    }
}
