use macroquad::prelude::*;

use crate::consts::*;

pub fn draw_body(scale_factor: usize, x: usize, y: usize, w: usize, h: usize) {
    let scale_factor = scale_factor as f32;
    let x = x as f32 * scale_factor;
    let y = y as f32 * scale_factor;
    let w = w as f32 * scale_factor;
    let h = h as f32 * scale_factor;
    draw_rectangle(x, y, w, h, UI_OUTER_BORDER_COLOR);
    draw_rectangle(
        x + 1.0 * scale_factor,
        y + 1.0 * scale_factor,
        w - 2.0 * scale_factor,
        h - 2.0 * scale_factor,
        UI_INNER_BORDER_COLOR,
    );
    draw_rectangle(
        x + 2.0 * scale_factor,
        y + 2.0 * scale_factor,
        w - 4.0 * scale_factor,
        h - 4.0 * scale_factor,
        UI_BG_COLOR,
    );
}
