use tauri::{LogicalSize, PhysicalPosition, WebviewWindow};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub const MARGIN: f64 = 12.0;
pub const WIDTH: f64 = 300.0;

const MIN_HEIGHT: f64 = 160.0;

/// Canto superior direito da área útil (já descontada a barra de tarefas, esteja ela onde estiver).
pub fn anchored_top_right(area: Rect, w: i32, margin: i32) -> (i32, i32) {
    (area.x + area.w - w - margin, area.y + margin)
}

/// Centralizado no ícone da bandeja, acima dele (caso comum: barra de tarefas embaixo). Se não
/// couber em cima (barra de tarefas no topo), cai pra baixo do ícone. Sempre dentro da área útil.
pub fn position_near_tray(tray: Rect, popup_w: i32, popup_h: i32, area: Rect, margin: i32) -> (i32, i32) {
    let cx = tray.x + tray.w / 2;
    let x = (cx - popup_w / 2).clamp(area.x, area.x + area.w - popup_w);
    let above = tray.y - popup_h - margin;
    let y = if above >= area.y { above } else { tray.y + tray.h + margin };
    (x, y)
}

/// Altura (px lógicos) que cabe na área útil do monitor, com margem em cima e embaixo.
pub fn fit_height(requested: f64, area_h_px: i32, scale: f64) -> f64 {
    let max = (f64::from(area_h_px) / scale - 2.0 * MARGIN).max(MIN_HEIGHT);
    requested.clamp(MIN_HEIGHT, max)
}

pub(crate) fn work_area(win: &WebviewWindow) -> Option<Rect> {
    let m = win
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| win.primary_monitor().ok().flatten())?;
    let wa = m.work_area();
    Some(Rect { x: wa.position.x, y: wa.position.y, w: wa.size.width as i32, h: wa.size.height as i32 })
}

/// Cola no canto superior direito da área útil do monitor atual. Só mexe se estiver fora do
/// lugar (reposicionar no mesmo ponto dispararia `Moved` de novo e entraria em laço).
pub fn anchor(win: &WebviewWindow) {
    let (Some(area), Ok(size), Ok(pos)) = (work_area(win), win.outer_size(), win.outer_position()) else { return };
    let margin = (MARGIN * win.scale_factor().unwrap_or(1.0)).round() as i32;
    let (x, y) = anchored_top_right(area, size.width as i32, margin);
    if (x, y) != (pos.x, pos.y) {
        let _ = win.set_position(PhysicalPosition::new(x, y));
    }
}

/// Fica no fundo da área de trabalho, atrás de todas as janelas.
pub fn sink(win: &WebviewWindow) {
    let _ = win.set_always_on_bottom(true);
}

/// Traz pra frente (clique no ícone da bandeja). Volta ao fundo sozinho quando perde o foco.
pub fn peek(win: &WebviewWindow) {
    let _ = win.set_always_on_bottom(false);
    let _ = win.show();
    let _ = win.set_focus();
}

/// Acrílico do Windows 11; se falhar, o CSS já tem fundo próprio.
pub fn apply_effects(win: &WebviewWindow) {
    #[cfg(target_os = "windows")]
    {
        let _ = window_vibrancy::apply_acrylic(win, Some((24, 28, 34, 180)));
    }
}

/// Pede ao WebView2 o modo de memória baixa (recomendado pela Microsoft para widgets que ficam
/// parados em segundo plano). Precisa do runtime ≥ 1.0.1774; se não suportar, não faz nada.
/// Medido em 2026-10-03: −18 MB de memória privada (−7%).
pub fn low_memory(win: &WebviewWindow) {
    #[cfg(target_os = "windows")]
    {
        let _ = win.with_webview(|wv| unsafe {
            use webview2_com::Microsoft::Web::WebView2::Win32::{
                ICoreWebView2_19, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
            };
            use windows_core::Interface;
            if let Ok(core) = wv.controller().CoreWebView2() {
                if let Ok(core19) = core.cast::<ICoreWebView2_19>() {
                    let _ = core19.SetMemoryUsageTargetLevel(COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW);
                }
            }
        });
    }
}

/// O frontend informa a altura do conteúdo; a janela cresce para baixo (topo fixo no canto).
#[tauri::command]
pub fn set_window_height(window: WebviewWindow, height: f64) -> Result<(), String> {
    let scale = window.scale_factor().unwrap_or(1.0);
    let height = match work_area(&window) {
        Some(area) => fit_height(height, area.h, scale),
        None => height.clamp(MIN_HEIGHT, 900.0),
    };
    window.set_size(LogicalSize::new(WIDTH, height)).map_err(|e| e.to_string())?;
    anchor(&window);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: i32 = 300;

    #[test]
    fn anchors_top_right_with_bottom_taskbar() {
        let area = Rect { x: 0, y: 0, w: 1920, h: 1032 }; // taskbar de 48px embaixo
        assert_eq!(anchored_top_right(area, W, 12), (1920 - 300 - 12, 12));
    }

    #[test]
    fn anchors_top_right_below_a_top_taskbar_and_on_offset_monitors() {
        let top_bar = Rect { x: 0, y: 48, w: 1920, h: 1032 };
        assert_eq!(anchored_top_right(top_bar, W, 12), (1608, 48 + 12));
        let second = Rect { x: 1920, y: -200, w: 1280, h: 984 };
        assert_eq!(anchored_top_right(second, W, 12), (1920 + 1280 - 312, -200 + 12));
    }

    #[test]
    fn positions_above_a_tray_icon_centered_with_bottom_taskbar() {
        let area = Rect { x: 0, y: 0, w: 1920, h: 1032 };
        let tray = Rect { x: 900, y: 1012, w: 20, h: 20 }; // longe da borda, não clampa
        assert_eq!(position_near_tray(tray, 280, 150, area, 8), (910 - 140, 1012 - 150 - 8));
    }

    #[test]
    fn position_near_tray_clamps_horizontally_at_the_screen_edge() {
        let area = Rect { x: 0, y: 0, w: 1920, h: 1032 };
        let tray = Rect { x: 1910, y: 1012, w: 20, h: 20 }; // ícone quase colado na borda direita
        assert_eq!(position_near_tray(tray, 280, 150, area, 8).0, 1920 - 280);
    }

    #[test]
    fn position_near_tray_falls_below_when_it_would_not_fit_above() {
        // barra de tarefas no topo: não cabe acima do ícone, cai pra baixo
        let area = Rect { x: 0, y: 48, w: 1920, h: 1032 };
        let tray = Rect { x: 100, y: 48, w: 20, h: 20 };
        // x também clampa (110-140 = -30, estoura a esquerda)
        assert_eq!(position_near_tray(tray, 280, 150, area, 8), (0, 48 + 20 + 8));
    }

    #[test]
    fn webview_runs_without_gpu_and_keeps_tauri_defaults() {
        // sem GPU: −50 MB de RAM, visual idêntico e menos CPU parado (medido em 2026-10-03)
        let conf: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        let args = conf["app"]["windows"][0]["additionalBrowserArgs"].as_str().unwrap_or("");
        assert!(args.contains("--disable-gpu"), "args: {args:?}");
        // definir additionalBrowserArgs substitui os padrões do Tauri — eles precisam continuar lá
        assert!(args.contains("--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection"), "args: {args:?}");
    }

    #[test]
    fn fit_height_stays_inside_the_work_area() {
        assert_eq!(fit_height(540.0, 1032, 1.0), 540.0);
        // conteúdo maior que a tela: sobra a margem em cima e embaixo
        assert_eq!(fit_height(900.0, 600, 1.0), 600.0 - 2.0 * MARGIN);
        // com escala 150% a área útil tem menos pixels lógicos
        assert_eq!(fit_height(900.0, 600, 1.5), 600.0 / 1.5 - 2.0 * MARGIN);
        // nunca encolhe abaixo do mínimo
        assert_eq!(fit_height(50.0, 1032, 1.0), 160.0);
        assert_eq!(fit_height(900.0, 100, 1.0), 160.0);
    }
}
