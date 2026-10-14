mod commands;
mod database;
mod error;
mod models;
mod services;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let project_dir = if cfg!(debug_assertions) {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .unwrap()
                    .to_path_buf()
            } else {
                std::env::current_exe()?.parent().unwrap().to_path_buf()
            };
            let runtime_root = if cfg!(debug_assertions) {
                project_dir
            } else {
                data_dir.clone()
            };
            let state = services::AppState::new(data_dir, runtime_root)
                .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_voices,
            commands::refresh_voices,
            commands::select_voice,
            commands::save_voice_settings,
            commands::open_voices_folder,
            commands::transcribe_audio,
            commands::service_health,
            commands::direct_voice_test,
            commands::latency_history,
            commands::save_api_key,
            commands::api_key_configured,
            commands::recent_chat,
            commands::send_chat,
            commands::initialize_memory_embeddings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
