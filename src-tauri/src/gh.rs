use std::process::Command;

use crate::cache;
use crate::types::PrListItem;
use crate::validation::validate_repo;

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
        "number,title,author,updatedAt,url,headRefName,baseRefName,reviewDecision,body".to_string(),
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
pub async fn get_pr_diff(app: tauri::AppHandle, repo: String, pr_number: u32) -> Result<String, String> {
    use tauri::Manager;
    validate_repo(&repo)?;

    let app_data_dir = app.path().app_data_dir().ok();
    let cache_key = format!("{}__{}", repo.replace('/', "__"), pr_number);

    // Check cache
    if let Some(ref dir) = app_data_dir {
        if let Some(cached) = cache::read_cache::<String>(dir, "cache/diff", &cache_key) {
            return Ok(cached);
        }
    }

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

    // Write cache
    if let Some(ref dir) = app_data_dir {
        cache::write_cache(dir, "cache/diff", &cache_key, &diff);
    }

    Ok(diff)
}
