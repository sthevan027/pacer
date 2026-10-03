pub mod alerts;
pub mod commands;
pub mod config;
pub mod pacing;
pub mod providers;
pub mod scheduler;
pub mod snapshot;
pub mod state;
pub mod tray;

use std::sync::Arc;
use tauri::{Manager, WindowEvent};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .setup(|app| {
            let config_path = app.path().app_config_dir()?.join("config.json");
            let cfg = config::load(&config_path);
            let shared = Arc::new(state::Shared::new(config_path, cfg.clone()));
            app.manage(shared.clone());

            let win = app.get_webview_window("main").expect("janela main");
            let w = win.clone();
            win.on_window_event(move |e| {
                if let WindowEvent::CloseRequested { api, .. } = e {
                    api.prevent_close();
                    let _ = w.hide();
                }
            });

            commands::apply_config(app.handle(), &cfg);
            win.show()?;
            scheduler::spawn(app.handle().clone(), shared);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::save_config,
            commands::refresh_now,
            commands::hide_window,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Pacer");
}
