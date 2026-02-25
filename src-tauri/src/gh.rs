use std::process::Command;

use crate::types::{PrListItem};

fn gh_env() -> Vec<(&'static str, &'static str)> {
    vec![
        ("GH_PAGER", "cat"),
        ("PAGER", "cat"),
        ("NO_COLOR", "1"),
        ("GH_FORCE_TTY", "0"),
    ]
}

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
                "GitHub CLI is not authenticated. Please run: gh auth login".to_string(),
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
