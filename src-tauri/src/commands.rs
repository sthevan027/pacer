use crate::config::{self, Config};
use crate::snapshot::Snapshot;
use crate::state::Shared;
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State, WebviewWindow};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub snapshots: Vec<Snapshot>,
    pub config: Config,
    pub version: String,
}

#[tauri::command]
pub fn get_state(app: AppHandle, shared: State<'_, Arc<Shared>>) -> AppState {
    AppState {
        snapshots: shared.snapshots.lock().unwrap().clone(),
        config: shared.config.lock().unwrap().clone(),
        version: app.package_info().version.to_string(),
    }
}

#[tauri::command]
pub fn save_config(app: AppHandle, shared: State<'_, Arc<Shared>>, config: Config) -> Result<Config, String> {
    let cfg = config.normalized();
    config::save(&shared.config_path, &cfg).map_err(|e| e.to_string())?;
    *shared.config.lock().unwrap() = cfg.clone();
    apply_config(&app, &cfg);
    let _ = app.emit("config", &cfg);
    shared.wake_up();
    Ok(cfg)
}

#[tauri::command]
pub fn refresh_now(shared: State<'_, Arc<Shared>>) {
    shared.request_refresh();
}

#[tauri::command]
pub fn hide_window(window: WebviewWindow) {
    let _ = window.hide();
}

/// Aplica efeitos colaterais da config (início com o Windows; trava na Task 11).
pub fn apply_config(app: &AppHandle, cfg: &Config) {
    if !cfg!(debug_assertions) {
        use tauri_plugin_autostart::ManagerExt;
        let launcher = app.autolaunch();
        let _ = if cfg.start_with_windows { launcher.enable() } else { launcher.disable() };
    }
}
