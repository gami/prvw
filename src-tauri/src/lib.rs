mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_prs,
            commands::get_pr_diff,
            commands::parse_diff,
            commands::split_large_hunks,
            commands::analyze_intents_with_codex,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
