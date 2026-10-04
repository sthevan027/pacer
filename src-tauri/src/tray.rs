use crate::snapshot::Snapshot;
use std::f64::consts::TAU;
use std::sync::Mutex;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

pub const SIZE: u32 = 32;
const TRACK: [u8; 3] = [0x6e, 0x76, 0x81];
const BLUE: [u8; 3] = [0x1f, 0x6f, 0xeb];
const ORANGE: [u8; 3] = [0xd2, 0x99, 0x22];
const RED: [u8; 3] = [0xf8, 0x51, 0x49];

/// Mesma escala das barras (src/lib/severity.ts): azul < 70%, laranja→vermelho de 70 a 90%,
/// vermelho ≥ 90%.
pub fn usage_color(pct: f64) -> [u8; 3] {
    if pct < 70.0 {
        return BLUE;
    }
    if pct >= 90.0 {
        return RED;
    }
    let t = (pct - 70.0) / 20.0;
    let mix = |a: u8, b: u8| (f64::from(a) + (f64::from(b) - f64::from(a)) * t).round() as u8;
    [mix(ORANGE[0], RED[0]), mix(ORANGE[1], RED[1]), mix(ORANGE[2], RED[2])]
}

/// Anel 32×32: trilho cinza + arco colorido proporcional ao uso, horário a partir do topo.
pub fn render_icon(pct: f64) -> Vec<u8> {
    let mut px = vec![0u8; (SIZE * SIZE * 4) as usize];
    let c = (SIZE as f64 - 1.0) / 2.0;
    let (r_out, r_in) = (15.0, 10.0);
    let frac = (pct / 100.0).clamp(0.0, 1.0);
    let col = usage_color(pct);
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

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let refresh = MenuItem::with_id(app, "refresh", "Atualizar agora", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Configurações", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&refresh, &settings, &sep, &quit])?;

    TrayIconBuilder::with_id("main")
        .icon(Image::new_owned(render_icon(0.0), SIZE, SIZE))
        .tooltip("Pacer")
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
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                // Sempre traz pra frente. Não dá pra alternar por `is_visible()`: no fundo da área de
                // trabalho o widget conta como visível mesmo coberto, e o 1º clique o escondia.
                // Esconder: Esc. Voltar ao fundo: clicar em outra coisa (window::sink).
                show_main(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

static LAST: Mutex<Option<u8>> = Mutex::new(None);

/// Mostra a maior % entre as janelas; troca o ícone só quando a % arredondada muda.
pub fn update(app: &AppHandle, snaps: &[Snapshot]) {
    let pct = snaps
        .iter()
        .flat_map(|s| s.windows.iter())
        .map(|w| w.used_pct)
        .fold(0.0_f64, f64::max);
    let key = pct.round() as u8;
    let mut last = LAST.lock().unwrap();
    if *last == Some(key) {
        return;
    }
    *last = Some(key);
    if let Some(t) = app.tray_by_id("main") {
        let _ = t.set_icon(Some(Image::new_owned(render_icon(pct), SIZE, SIZE)));
        let _ = t.set_tooltip(Some(format!("Pacer · {key}%")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn px(buf: &[u8], x: u32, y: u32) -> [u8; 4] {
        let i = ((y * SIZE + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    #[test]
    fn usage_color_blue_then_orange_to_red_then_red() {
        assert_eq!(usage_color(0.0), [0x1f, 0x6f, 0xeb]);
        assert_eq!(usage_color(69.9), [0x1f, 0x6f, 0xeb]);
        assert_eq!(usage_color(70.0), [0xd2, 0x99, 0x22]);
        // no meio do caminho: média entre laranja e vermelho
        assert_eq!(usage_color(80.0), [229, 117, 54]);
        assert_eq!(usage_color(90.0), [0xf8, 0x51, 0x49]);
        assert_eq!(usage_color(100.0), [0xf8, 0x51, 0x49]);
    }

    #[test]
    fn ring_fills_clockwise_from_top() {
        let buf = render_icon(50.0);
        assert_eq!(buf.len(), (SIZE * SIZE * 4) as usize);
        assert_eq!(px(&buf, 16, 2), [0x1f, 0x6f, 0xeb, 255]); // topo, logo depois do 0° → preenchido
        assert_eq!(px(&buf, 2, 16), [0x6e, 0x76, 0x81, 255]); // esquerda (~270°) → trilho cinza
        assert_eq!(px(&buf, 16, 16)[3], 0); // centro transparente
        assert_eq!(px(&buf, 0, 0)[3], 0); // canto transparente
    }

    #[test]
    fn ring_color_follows_usage() {
        assert_eq!(px(&render_icon(95.0), 16, 2), [0xf8, 0x51, 0x49, 255]);
        assert_eq!(px(&render_icon(80.0), 16, 2), [229, 117, 54, 255]);
        assert_eq!(px(&render_icon(30.0), 16, 2), [0x1f, 0x6f, 0xeb, 255]);
    }

    #[test]
    fn zero_percent_is_all_track() {
        assert_eq!(px(&render_icon(0.0), 16, 2), [0x6e, 0x76, 0x81, 255]);
    }
}
