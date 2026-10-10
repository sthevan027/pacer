use crate::config::{self, Config};
use crate::snapshot::Snapshot;
use crate::state::Shared;
use crate::usage::{report_csv, ReportFilter, UsageReport};
use chrono::Local;
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

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
    // Cor de destaque: redesenha o ícone já, sem esperar o próximo refresh do scheduler.
    crate::tray::update(&app, &shared.snapshots.lock().unwrap(), &cfg.accent_color);
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

/// Abre o painel de uso (botão no widget; o menu da bandeja chama `panel::open` direto).
#[tauri::command]
pub fn open_panel(app: AppHandle) {
    crate::panel::open(&app);
}

/// Consumo detalhado do período, calculado sob demanda — nada disso entra no `Snapshot`, que é
/// empurrado ao widget a cada refresh e ficaria pesado.
#[tauri::command]
pub async fn get_usage_report(
    shared: State<'_, Arc<Shared>>,
    range_days: i64,
    families: Vec<String>,
) -> Result<UsageReport, String> {
    let shared = shared.inner().clone();
    let usd_brl = shared.config.lock().unwrap().usd_brl;
    // Agregar 30 dias de eventos é rápido, mas não pode travar a thread principal.
    tauri::async_runtime::spawn_blocking(move || {
        let store = shared.usage.lock().unwrap();
        store.report(&ReportFilter { range_days, families }, Local::now(), usd_brl)
    })
    .await
    .map_err(|e| e.to_string())
}

/// Grava o CSV das requisições em Downloads e devolve o caminho.
/// Contém nomes de projeto e de sessão — é o disco do usuário, nada sai da máquina.
#[tauri::command]
pub async fn export_usage_csv(
    app: AppHandle,
    shared: State<'_, Arc<Shared>>,
    range_days: i64,
    families: Vec<String>,
) -> Result<String, String> {
    let shared = shared.inner().clone();
    let usd_brl = shared.config.lock().unwrap().usd_brl;
    let dir = app.path().download_dir().map_err(|e| e.to_string())?;

    let csv = tauri::async_runtime::spawn_blocking(move || {
        let store = shared.usage.lock().unwrap();
        report_csv(&store.report(&ReportFilter { range_days, families }, Local::now(), usd_brl))
    })
    .await
    .map_err(|e| e.to_string())?;

    let path = dir.join(format!("pacer-uso-{}.csv", Local::now().format("%Y-%m-%d")));
    // BOM: nome de projeto pode ter acento, e sem isso o Excel abre errado.
    let mut bytes = Vec::with_capacity(csv.len() + 3);
    bytes.extend_from_slice("\u{feff}".as_bytes());
    bytes.extend_from_slice(csv.as_bytes());
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().into_owned())
}

/// Aplica efeitos colaterais da config (início com o Windows).
pub fn apply_config(app: &AppHandle, cfg: &Config) {
    if !cfg!(debug_assertions) {
        use tauri_plugin_autostart::ManagerExt;
        let launcher = app.autolaunch();
        let _ = if cfg.start_with_windows { launcher.enable() } else { launcher.disable() };
    }
}
