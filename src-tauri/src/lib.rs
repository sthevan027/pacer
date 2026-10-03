pub mod pacing;
pub mod snapshot;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            use tauri::Manager;
            app.get_webview_window("main").expect("janela main").show()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Pacer");
}
