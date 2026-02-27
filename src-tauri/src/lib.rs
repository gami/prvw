mod codex;
mod codex_runner;
mod diff_parser;
mod gh;
mod types;
mod validation;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            gh::list_prs,
            gh::get_pr_diff,
            diff_parser::parse_diff,
            codex::analyze_intents_with_codex,
            codex::refine_group,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
