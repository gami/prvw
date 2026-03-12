use std::process::Command;

use crate::cache;
use crate::types::PrListItem;

fn validate_repo(repo: &str) -> Result<(), String> {
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2
        || parts[0].is_empty()
        || parts[1].is_empty()
        || parts
            .iter()
            .any(|p| p.contains(|c: char| c.is_whitespace()))
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
        "number,title,author,updatedAt,url,headRefName,baseRefName,reviewDecision,isDraft,body"
            .to_string(),
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
            return Err("GitHub CLI is not authenticated. Please run: gh auth login".to_string());
        }
        return Err(format!("gh pr list failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let items: Vec<PrListItem> =
        serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse gh output: {}", e))?;
    Ok(items)
}

#[tauri::command]
pub async fn get_pr_diff(
    app: tauri::AppHandle,
    repo: String,
    pr_number: u32,
    updated_at: Option<String>,
    force: Option<bool>,
) -> Result<String, String> {
    use tauri::Manager;
    validate_repo(&repo)?;

    let app_data_dir = app.path().app_data_dir().ok();
    let ts = updated_at.as_deref().unwrap_or("").replace(':', "-");
    let cache_key = format!("{}__{}_{}", repo.replace('/', "__"), pr_number, ts);

    // Check cache (unless force)
    if force != Some(true) {
        if let Some(ref dir) = app_data_dir {
            if let Some(cached) = cache::read_cache::<String>(dir, "cache/diff", &cache_key) {
                return Ok(cached);
            }
        }
    }

    let output = Command::new("gh")
        .args([
            "pr",
            "diff",
            "-R",
            &repo,
            &pr_number.to_string(),
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

    let diff = if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("too_large") || stderr.contains("HTTP 406") {
            // Diff too large for GitHub API — fall back to git diff via local clone
            get_pr_diff_via_git(&repo, pr_number)?
        } else {
            return Err(format!("gh pr diff failed: {}", stderr));
        }
    } else {
        String::from_utf8_lossy(&output.stdout).to_string()
    };
    if diff.trim().is_empty() {
        return Err("Diff is empty. The PR may have no changes.".to_string());
    }

    // Write cache
    if let Some(ref dir) = app_data_dir {
        cache::write_cache(dir, "cache/diff", &cache_key, &diff);
    }

    Ok(diff)
}

/// Fallback: fetch PR branch refs via gh, then use git diff against a local clone.
fn get_pr_diff_via_git(repo: &str, pr_number: u32) -> Result<String, String> {
    // Get head and base branch names from the PR metadata
    let meta_output = Command::new("gh")
        .args([
            "pr",
            "view",
            "-R",
            repo,
            &pr_number.to_string(),
            "--json",
            "headRefName,baseRefName",
        ])
        .envs(gh_env())
        .output()
        .map_err(|e| format!("Failed to execute gh pr view: {}", e))?;

    if !meta_output.status.success() {
        let stderr = String::from_utf8_lossy(&meta_output.stderr);
        return Err(format!("gh pr view failed: {}", stderr));
    }

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct PrMeta {
        head_ref_name: String,
        base_ref_name: String,
    }

    let meta: PrMeta = serde_json::from_slice(&meta_output.stdout)
        .map_err(|e| format!("Failed to parse PR metadata: {}", e))?;

    // Clone (shallow, bare) into a temp dir and diff
    let temp_dir = tempfile::tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let clone_path = temp_dir.path().join("repo");

    let clone_status = Command::new("gh")
        .args([
            "repo",
            "clone",
            repo,
            clone_path
                .to_str()
                .ok_or_else(|| "Non-UTF-8 temp path".to_string())?,
            "--",
            "--bare",
            "--filter=blob:none",
        ])
        .envs(gh_env())
        .status()
        .map_err(|e| format!("Failed to clone repo: {}", e))?;

    if !clone_status.success() {
        return Err("Failed to clone repository for large diff fallback.".to_string());
    }

    let clone_str = clone_path
        .to_str()
        .ok_or_else(|| "Non-UTF-8 clone path".to_string())?;

    // Fetch both branches explicitly (bare clone may not have all refs)
    let _ = Command::new("git")
        .args([
            "-C",
            clone_str,
            "fetch",
            "origin",
            &format!(
                "+refs/heads/{}:refs/heads/{} +refs/heads/{}:refs/heads/{}",
                meta.base_ref_name, meta.base_ref_name, meta.head_ref_name, meta.head_ref_name
            ),
        ])
        .output();

    let diff_output = Command::new("git")
        .args([
            "-C",
            clone_str,
            "diff",
            &format!("{}...{}", meta.base_ref_name, meta.head_ref_name),
        ])
        .output()
        .map_err(|e| format!("git diff failed: {}", e))?;

    if !diff_output.status.success() {
        let stderr = String::from_utf8_lossy(&diff_output.stderr);
        return Err(format!("git diff failed for large PR: {}", stderr));
    }

    Ok(String::from_utf8_lossy(&diff_output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_repo_valid() {
        assert!(validate_repo("owner/repo").is_ok());
    }

    #[test]
    fn validate_repo_no_slash() {
        assert!(validate_repo("noslash").is_err());
    }

    #[test]
    fn validate_repo_empty_owner() {
        assert!(validate_repo("/repo").is_err());
    }

    #[test]
    fn validate_repo_empty_name() {
        assert!(validate_repo("owner/").is_err());
    }

    #[test]
    fn validate_repo_whitespace_in_parts() {
        assert!(validate_repo("ow ner/repo").is_err());
        assert!(validate_repo("owner/re po").is_err());
    }

    #[test]
    fn validate_repo_too_many_slashes() {
        assert!(validate_repo("a/b/c").is_err());
    }
}
