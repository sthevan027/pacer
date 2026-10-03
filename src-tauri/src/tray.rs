use crate::snapshot::{severity, Severity, Snapshot};
use std::f64::consts::TAU;
use std::sync::Mutex;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

pub const SIZE: u32 = 32;
const TRACK: [u8; 3] = [0x6e, 0x76, 0x81];

fn color(s: Severity) -> [u8; 3] {
    match s {
        Severity::Ok => [0x1f, 0x6f, 0xeb],
        Severity::Warn => [0xd2, 0x99, 0x22],
        Severity::Critical => [0xf8, 0x51, 0x49],
    }
}

/// Anel 32×32: trilho cinza + arco colorido proporcional ao uso, horário a partir do topo.
pub fn render_icon(pct: f64, sev: Severity) -> Vec<u8> {
    let mut px = vec![0u8; (SIZE * SIZE * 4) as usize];
    let c = (SIZE as f64 - 1.0) / 2.0;
    let (r_out, r_in) = (15.0, 10.0);
    let frac = (pct / 100.0).clamp(0.0, 1.0);
    let col = color(sev);
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
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn toggle_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
        } else {
            show_main(app);
        }
    }
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let refresh = MenuItem::with_id(app, "refresh", "Atualizar agora", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Configurações", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&refresh, &settings, &sep, &quit])?;

    TrayIconBuilder::with_id("main")
        .icon(Image::new_owned(render_icon(0.0, Severity::Ok), SIZE, SIZE))
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
                toggle_main(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

static LAST: Mutex<Option<(u8, Severity)>> = Mutex::new(None);

/// Troca o ícone só quando a % arredondada ou a severidade mudam.
pub fn update(app: &AppHandle, snaps: &[Snapshot]) {
    let (pct, sev) = snaps
        .iter()
        .flat_map(|s| s.windows.iter())
        .map(|w| (w.used_pct, severity(w.used_pct, w.pace.as_ref())))
        .fold((0.0_f64, Severity::Ok), |(p, s), (wp, ws)| (p.max(wp), s.max(ws)));
    let key = (pct.round() as u8, sev);
    let mut last = LAST.lock().unwrap();
    if *last == Some(key) {
        return;
    }
    *last = Some(key);
    if let Some(t) = app.tray_by_id("main") {
        let _ = t.set_icon(Some(Image::new_owned(render_icon(pct, sev), SIZE, SIZE)));
        let _ = t.set_tooltip(Some(format!("Pacer · {}%", key.0)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::Severity;

    fn px(buf: &[u8], x: u32, y: u32) -> [u8; 4] {
        let i = ((y * SIZE + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    #[test]
    fn ring_fills_clockwise_from_top() {
        let buf = render_icon(50.0, Severity::Ok);
        assert_eq!(buf.len(), (SIZE * SIZE * 4) as usize);
        assert_eq!(px(&buf, 16, 2), [0x1f, 0x6f, 0xeb, 255]); // topo, logo depois do 0° → preenchido
        assert_eq!(px(&buf, 2, 16), [0x6e, 0x76, 0x81, 255]); // esquerda (~270°) → trilho cinza
        assert_eq!(px(&buf, 16, 16)[3], 0); // centro transparente
        assert_eq!(px(&buf, 0, 0)[3], 0); // canto transparente
    }

    #[test]
    fn color_follows_severity() {
        assert_eq!(px(&render_icon(95.0, Severity::Critical), 16, 2), [0xf8, 0x51, 0x49, 255]);
        assert_eq!(px(&render_icon(80.0, Severity::Warn), 16, 2), [0xd2, 0x99, 0x22, 255]);
    }

    #[test]
    fn zero_percent_is_all_track() {
        assert_eq!(px(&render_icon(0.0, Severity::Ok), 16, 2), [0x6e, 0x76, 0x81, 255]);
    }
}
