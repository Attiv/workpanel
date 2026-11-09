mod ai;
mod api;
mod db;
mod kanri;
mod model;
mod report;

#[cfg(test)]
mod tests;

use api::AppState;
use std::sync::Mutex;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let db_path = match std::env::var("WORKPANEL_DB") {
                Ok(p) if !p.trim().is_empty() => std::path::PathBuf::from(p),
                _ => dir.join("workpanel.db"),
            };
            let conn = db::open(&db_path)?;

            // First launch on an empty database: pull in the Kanri board so the
            // panel opens with the user's real tasks instead of nothing.
            let empty: i64 = conn
                .query_row("SELECT (SELECT COUNT(*) FROM tasks) + (SELECT COUNT(*) FROM projects)", [], |r| r.get(0))
                .unwrap_or(1);
            if empty == 0 && kanri::default_path().exists() {
                let _ = kanri::import(&conn, None);
            }

            #[cfg(debug_assertions)]
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.set_focus();
                let _ = win.set_always_on_top(true);
            }

            app.manage(AppState {
                conn: Mutex::new(conn),
                db_path: db_path.display().to_string(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            api::snapshot,
            api::app_info,
            api::save_project,
            api::delete_project,
            api::save_task,
            api::delete_task,
            api::set_task_status,
            api::set_task_plan,
            api::start_timer,
            api::stop_timer,
            api::add_entry,
            api::update_entry,
            api::delete_entry,
            api::build_report,
            api::save_settings,
            api::import_kanri,
            api::save_text_file,
            api::write_temp_html,
            api::translate_missing,
            api::polish_text,
            api::test_ai
        ])
        .run(tauri::generate_context!())
        .expect("error while running WorkPanel");
}
