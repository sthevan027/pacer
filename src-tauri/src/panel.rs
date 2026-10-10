//! Janela do painel de uso.
//!
//! Ao contrário do widget (`main`), que fica vivo o tempo todo no canto, esta é de uso
//! esporádico: nasce no primeiro clique e morre ao fechar, devolvendo o WebView2. Também é a
//! única janela decorada e redimensionável do app, então aparece na barra de tarefas.

use crate::tray::BROWSER_ARGS;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::webview::WebviewWindowBuilder;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow};

pub const LABEL: &str = "panel";

const W: f64 = 980.0;
const H: f64 = 680.0;
const MIN_W: f64 = 720.0;
const MIN_H: f64 = 460.0;

/// Evita duas threads construindo a janela ao mesmo tempo (dois cliques seguidos).
static BUILDING: AtomicBool = AtomicBool::new(false);

fn build(app: &AppHandle) -> Option<WebviewWindow> {
    let win = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html?view=panel".into()))
        .title("Pacer — Uso")
        .inner_size(W, H)
        .min_inner_size(MIN_W, MIN_H)
        .resizable(true)
        .decorations(true)
        .additional_browser_args(BROWSER_ARGS)
        .build()
        .ok()?;
    crate::window::low_memory(&win);
    // Sem acrílico: aqui a janela é opaca e decorada, o fundo sólido vem do CSS.
    Some(win)
}

/// Abre o painel, ou traz pra frente se já estiver aberto.
///
/// **Nunca** chame [`build`] direto daqui: os dois chamadores (item do menu da bandeja e comando
/// vindo do widget) acabam na thread principal, e `WebviewWindowBuilder::build` dá deadlock no
/// Windows nessa situação — o WebView2 nunca termina de nascer e a thread principal trava,
/// deixando o cursor de "carregando" na tela. Mesma lição do popup de hover (`tray::show_hover`).
pub fn open(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(LABEL) {
        let _ = win.unminimize();
        let _ = win.show();
        let _ = win.set_focus();
        return;
    }
    if BUILDING.swap(true, Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        let win = build(&app);
        BUILDING.store(false, Ordering::SeqCst);
        if let Some(win) = win {
            let _ = win.show();
            let _ = win.set_focus();
        }
    });
}
