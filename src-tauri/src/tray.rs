use crate::snapshot::Snapshot;
use crate::window::{self, Rect};
use std::f64::consts::TAU;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::webview::WebviewWindowBuilder;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindow};

/// Mesmos argumentos do WebView2 da janela `main` (ver `tauri.conf.json`): sem GPU, memória
/// mais baixa parada em segundo plano.
const BROWSER_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --disable-gpu";

/// Janelinha visual que aparece ao passar o mouse na bandeja (Sessão + Semanal).
const HOVER_W: i32 = 280;
const HOVER_H: i32 = 150;
const HOVER_MARGIN: i32 = 8;
/// Quanto esperar depois do mouse sair antes de esconder (evita piscar ao passar rápido).
const HOVER_HIDE_DELAY: Duration = Duration::from_millis(250);

pub const SIZE: u32 = 32;
const TRACK: [u8; 3] = [0x6e, 0x76, 0x81];
/// Mesmo valor de `config::DEFAULT_ACCENT`: usado se o hex salvo não existir ou vier inválido
/// (não deveria acontecer, já que `Config::normalized` só aceita um dos presets).
const DEFAULT_ACCENT: [u8; 3] = [0x1f, 0x6f, 0xeb];
const ORANGE: [u8; 3] = [0xd2, 0x99, 0x22];
const RED: [u8; 3] = [0xf8, 0x51, 0x49];

/// `"#rrggbb"` → RGB. `None` se não tiver o formato esperado.
pub fn parse_hex_rgb(hex: &str) -> Option<[u8; 3]> {
    let h = hex.strip_prefix('#')?;
    if h.len() != 6 {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok();
    Some([byte(0)?, byte(2)?, byte(4)?])
}

/// Mesma escala das barras (src/lib/severity.ts): a cor de destaque < 70%, laranja→vermelho de
/// 70 a 90%, vermelho ≥ 90% (aviso/crítico nunca mudam, só o estado "ok" é customizável).
pub fn usage_color(pct: f64, accent: [u8; 3]) -> [u8; 3] {
    if pct < 70.0 {
        return accent;
    }
    if pct >= 90.0 {
        return RED;
    }
    let t = (pct - 70.0) / 20.0;
    let mix = |a: u8, b: u8| (f64::from(a) + (f64::from(b) - f64::from(a)) * t).round() as u8;
    [mix(ORANGE[0], RED[0]), mix(ORANGE[1], RED[1]), mix(ORANGE[2], RED[2])]
}

/// Anel 32×32: trilho cinza + arco colorido proporcional ao uso, horário a partir do topo.
pub fn render_icon(pct: f64, accent: [u8; 3]) -> Vec<u8> {
    let mut px = vec![0u8; (SIZE * SIZE * 4) as usize];
    let c = (SIZE as f64 - 1.0) / 2.0;
    let (r_out, r_in) = (15.0, 10.0);
    let frac = (pct / 100.0).clamp(0.0, 1.0);
    let col = usage_color(pct, accent);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (dx, dy) = (x as f64 - c, y as f64 - c);
            let d = (dx * dx + dy * dy).sqrt();
            if d > r_out || d < r_in {
                continue;
            }
            let ang = (dx.atan2(-dy) + TAU) % TAU;
            let rgb = if frac > 0.0 && ang / TAU <= frac { col } else { TRACK };
            let i = ((y * SIZE + x) * 4) as usize;
            px[i..i + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
    }
    px
}

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        crate::window::peek(&w);
    }
}

/// Monotônico: toda `Enter` ou `Leave` avança. Uma `hide` atrasada só executa se, quando chegar
/// a vez dela, ninguém tiver mexido no hover de novo nesse meio tempo (outro Enter, outra Leave).
static HOVER_GEN: AtomicU32 = AtomicU32::new(0);

/// A janela só existe depois do primeiro hover (não custa memória de um 2º WebView se o
/// usuário nunca passar o mouse ali). Depois de criada, fica escondida e é só mostrada/escondida.
fn hover_window(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(win) = app.get_webview_window("hover") {
        return Some(win);
    }
    let win = WebviewWindowBuilder::new(app, "hover", WebviewUrl::App("index.html?view=hover".into()))
        .title("Pacer")
        .inner_size(f64::from(HOVER_W), f64::from(HOVER_H))
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .focused(false)
        .additional_browser_args(BROWSER_ARGS)
        .build()
        .ok()?;
    // Sem acrílico aqui: combinado com set_ignore_cursor_events (clique-através), a composição
    // do Windows quebrava visualmente quando a janela main roubava o foco. O fundo translúcido
    // já vem do CSS (.wg), não depende do efeito nativo.
    window::low_memory(&win);
    // Só pra olhar: nunca intercepta clique (inclusive de ícones vizinhos na bandeja).
    let _ = win.set_ignore_cursor_events(true);
    Some(win)
}

/// Mostra a janelinha de hover posicionada perto do ícone, com as barras de Sessão e Semanal.
/// Não aparece se o painel principal já estiver em primeiro plano (evita sobrepor UI).
fn show_hover(app: &AppHandle, tray_rect: tauri::Rect) {
    HOVER_GEN.fetch_add(1, Ordering::SeqCst);
    if app.get_webview_window("main").is_some_and(|w| w.is_focused().unwrap_or(false)) {
        return;
    }
    let Some(win) = hover_window(app) else { return };
    let scale = win.scale_factor().unwrap_or(1.0);
    let pos = tray_rect.position.to_physical::<i32>(scale);
    let size = tray_rect.size.to_physical::<u32>(scale);
    let tray = Rect { x: pos.x, y: pos.y, w: size.width as i32, h: size.height as i32 };
    let Some(area) = window::work_area(&win) else { return };
    let margin = (f64::from(HOVER_MARGIN) * scale).round() as i32;
    let w = (f64::from(HOVER_W) * scale).round() as i32;
    let h = (f64::from(HOVER_H) * scale).round() as i32;
    let (x, y) = window::position_near_tray(tray, w, h, area, margin);
    let _ = win.set_position(PhysicalPosition::new(x, y));
    let _ = win.show();
}

/// Esconde o hover depois de `HOVER_HIDE_DELAY`, a menos que um `Enter` novo cancele antes.
fn hide_hover_later(app: &AppHandle) {
    let gen = HOVER_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(HOVER_HIDE_DELAY).await;
        if HOVER_GEN.load(Ordering::SeqCst) != gen {
            return;
        }
        if let Some(win) = app.get_webview_window("hover") {
            let _ = win.hide();
        }
    });
}

pub fn create(app: &AppHandle, accent_hex: &str) -> tauri::Result<()> {
    let refresh = MenuItem::with_id(app, "refresh", "Atualizar agora", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Configurações", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&refresh, &settings, &sep, &quit])?;
    let accent = parse_hex_rgb(accent_hex).unwrap_or(DEFAULT_ACCENT);

    TrayIconBuilder::with_id("main")
        .icon(Image::new_owned(render_icon(0.0, accent), SIZE, SIZE))
        // Sem tooltip nativo: o popup de hover (janela "hover") já mostra Sessão e Semanal.
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "refresh" => app.state::<std::sync::Arc<crate::state::Shared>>().request_refresh(),
            "settings" => {
                show_main(app);
                let _ = app.emit("open-settings", ());
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } => {
                // Sempre traz pra frente. Não dá pra alternar por `is_visible()`: no fundo da área de
                // trabalho o widget conta como visível mesmo coberto, e o 1º clique o escondia.
                // Esconder: Esc. Voltar ao fundo: clicar em outra coisa (window::sink).
                show_main(tray.app_handle());
            }
            TrayIconEvent::Enter { rect, .. } => show_hover(tray.app_handle(), rect),
            TrayIconEvent::Leave { .. } => hide_hover_later(tray.app_handle()),
            _ => {}
        })
        .build(app)?;
    Ok(())
}

static LAST: Mutex<Option<(u8, [u8; 3])>> = Mutex::new(None);

/// Mostra a maior % entre as janelas; troca o ícone só quando a % arredondada ou a cor mudam
/// (a cor muda ao salvar as Configurações, sem esperar o próximo refresh).
pub fn update(app: &AppHandle, snaps: &[Snapshot], accent_hex: &str) {
    let pct = snaps
        .iter()
        .flat_map(|s| s.windows.iter())
        .map(|w| w.used_pct)
        .fold(0.0_f64, f64::max);
    let accent = parse_hex_rgb(accent_hex).unwrap_or(DEFAULT_ACCENT);
    let key = (pct.round() as u8, accent);
    let mut last = LAST.lock().unwrap();
    if *last == Some(key) {
        return;
    }
    *last = Some(key);
    if let Some(t) = app.tray_by_id("main") {
        let _ = t.set_icon(Some(Image::new_owned(render_icon(pct, accent), SIZE, SIZE)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLUE: [u8; 3] = DEFAULT_ACCENT;
    const PURPLE: [u8; 3] = [0x89, 0x57, 0xe5];

    fn px(buf: &[u8], x: u32, y: u32) -> [u8; 4] {
        let i = ((y * SIZE + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    #[test]
    fn parses_hex_and_rejects_bad_formats() {
        assert_eq!(parse_hex_rgb("#1f6feb"), Some(BLUE));
        assert_eq!(parse_hex_rgb("1f6feb"), None); // sem #
        assert_eq!(parse_hex_rgb("#1f6fe"), None); // curto demais
        assert_eq!(parse_hex_rgb("#1f6fezz"), None); // não é hex
    }

    #[test]
    fn usage_color_accent_then_orange_to_red_then_red() {
        assert_eq!(usage_color(0.0, PURPLE), PURPLE);
        assert_eq!(usage_color(69.9, PURPLE), PURPLE);
        // aviso/crítico nunca mudam, qualquer que seja a cor de destaque
        assert_eq!(usage_color(70.0, PURPLE), [0xd2, 0x99, 0x22]);
        assert_eq!(usage_color(80.0, PURPLE), [229, 117, 54]); // meio do caminho laranja→vermelho
        assert_eq!(usage_color(90.0, PURPLE), [0xf8, 0x51, 0x49]);
        assert_eq!(usage_color(100.0, PURPLE), [0xf8, 0x51, 0x49]);
    }

    #[test]
    fn ring_fills_clockwise_from_top() {
        let buf = render_icon(50.0, BLUE);
        assert_eq!(buf.len(), (SIZE * SIZE * 4) as usize);
        assert_eq!(px(&buf, 16, 2), [0x1f, 0x6f, 0xeb, 255]); // topo, logo depois do 0° → preenchido
        assert_eq!(px(&buf, 2, 16), [0x6e, 0x76, 0x81, 255]); // esquerda (~270°) → trilho cinza
        assert_eq!(px(&buf, 16, 16)[3], 0); // centro transparente
        assert_eq!(px(&buf, 0, 0)[3], 0); // canto transparente
    }

    #[test]
    fn ring_color_follows_usage() {
        assert_eq!(px(&render_icon(95.0, BLUE), 16, 2), [0xf8, 0x51, 0x49, 255]);
        assert_eq!(px(&render_icon(80.0, BLUE), 16, 2), [229, 117, 54, 255]);
        assert_eq!(px(&render_icon(30.0, BLUE), 16, 2), [0x1f, 0x6f, 0xeb, 255]);
    }

    #[test]
    fn ring_ok_state_follows_the_accent_color() {
        assert_eq!(px(&render_icon(30.0, PURPLE), 16, 2), [0x89, 0x57, 0xe5, 255]);
        // crítico ignora a cor de destaque
        assert_eq!(px(&render_icon(95.0, PURPLE), 16, 2), [0xf8, 0x51, 0x49, 255]);
    }

    #[test]
    fn zero_percent_is_all_track() {
        assert_eq!(px(&render_icon(0.0, BLUE), 16, 2), [0x6e, 0x76, 0x81, 255]);
    }
}
