use macroquad::{color::BLACK, shapes::draw_rectangle};

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
pub fn draw_square(scale_factor: usize, x: usize, y: usize, w: usize, h: usize) {
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
        UI_BG_COLOR,
    );
}
pub fn draw_text(scale_factor: usize, x: usize, y: usize, text: &str) {
    let scale_factor = scale_factor as f32;
    let x = x as f32 * scale_factor;
    let y = y as f32 * scale_factor + 5.0;
    macroquad::text::draw_text(
        text,
        x * scale_factor,
        y * scale_factor,
        8.0 * scale_factor,
        BLACK,
    );
}
