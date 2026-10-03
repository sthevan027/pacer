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

pub fn anchored(area: Rect, w: i32, h: i32, margin: i32) -> (i32, i32) {
    (area.x + area.w - w - margin, area.y + area.h - h - margin)
}

pub fn clamp(area: Rect, x: i32, y: i32, w: i32, h: i32) -> (i32, i32) {
    let max_x = (area.x + area.w - w).max(area.x);
    let max_y = (area.y + area.h - h).max(area.y);
    (x.clamp(area.x, max_x), y.clamp(area.y, max_y))
}

fn work_area(win: &WebviewWindow) -> Option<Rect> {
    let m = win
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| win.primary_monitor().ok().flatten())?;
    let wa = m.work_area();
    Some(Rect { x: wa.position.x, y: wa.position.y, w: wa.size.width as i32, h: wa.size.height as i32 })
}

/// Cola no canto inferior direito da área útil do monitor atual.
pub fn anchor(win: &WebviewWindow) {
    let (Some(area), Ok(size)) = (work_area(win), win.outer_size()) else { return };
    let margin = (MARGIN * win.scale_factor().unwrap_or(1.0)).round() as i32;
    let (x, y) = anchored(area, size.width as i32, size.height as i32, margin);
    let _ = win.set_position(PhysicalPosition::new(x, y));
}

/// Nunca deixa a janela ficar fora da área útil (sob a taskbar ou fora da tela).
pub fn keep_inside(win: &WebviewWindow) {
    let (Some(area), Ok(pos), Ok(size)) = (work_area(win), win.outer_position(), win.outer_size()) else { return };
    let (x, y) = clamp(area, pos.x, pos.y, size.width as i32, size.height as i32);
    if (x, y) != (pos.x, pos.y) {
        let _ = win.set_position(PhysicalPosition::new(x, y));
    }
}

/// Acrílico do Windows 11; se falhar, o CSS já tem fundo próprio.
pub fn apply_effects(win: &WebviewWindow) {
    #[cfg(target_os = "windows")]
    {
        let _ = window_vibrancy::apply_acrylic(win, Some((24, 28, 34, 180)));
    }
}

/// O frontend informa a altura do conteúdo; a janela cresce para cima (base fixa).
#[tauri::command]
pub fn set_window_height(window: WebviewWindow, height: f64) -> Result<(), String> {
    let height = height.clamp(160.0, 900.0);
    let old_pos = window.outer_position().map_err(|e| e.to_string())?;
    let old_size = window.outer_size().map_err(|e| e.to_string())?;
    window.set_size(LogicalSize::new(WIDTH, height)).map_err(|e| e.to_string())?;
    let new_size = window.outer_size().map_err(|e| e.to_string())?;
    let bottom = old_pos.y + old_size.height as i32;
    let _ = window.set_position(PhysicalPosition::new(old_pos.x, bottom - new_size.height as i32));
    keep_inside(&window);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: i32 = 300;
    const H: i32 = 520;

    #[test]
    fn anchors_bottom_right_above_bottom_taskbar() {
        let area = Rect { x: 0, y: 0, w: 1920, h: 1032 }; // taskbar de 48px embaixo
        assert_eq!(anchored(area, W, H, 12), (1920 - 300 - 12, 1032 - 520 - 12));
    }

    #[test]
    fn respects_top_taskbar_and_offset_monitor() {
        let top_bar = Rect { x: 0, y: 48, w: 1920, h: 1032 };
        assert_eq!(anchored(top_bar, W, H, 12), (1608, 48 + 1032 - 520 - 12));
        let second = Rect { x: 1920, y: -200, w: 1280, h: 984 };
        assert_eq!(anchored(second, W, H, 12), (1920 + 1280 - 312, -200 + 984 - 532));
    }

    #[test]
    fn clamp_pulls_window_back_inside() {
        let area = Rect { x: 0, y: 0, w: 1920, h: 1032 };
        assert_eq!(clamp(area, 1800, 900, W, H), (1620, 512));
        assert_eq!(clamp(area, -50, -10, W, H), (0, 0));
        assert_eq!(clamp(area, 100, 100, W, H), (100, 100));
    }

    #[test]
    fn clamp_handles_window_bigger_than_area() {
        let area = Rect { x: 10, y: 10, w: 200, h: 200 };
        assert_eq!(clamp(area, 50, 50, W, H), (10, 10));
    }
}
