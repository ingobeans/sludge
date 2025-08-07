use macroquad::{color::BLACK, shapes::draw_rectangle};

use crate::consts::*;

pub fn draw_body(x: i16, y: i16, w: i16, h: i16) {
    let x = x as f32;
    let y = y as f32;
    let w = w as f32;
    let h = h as f32;
    draw_rectangle(x, y, w, h, UI_OUTER_BORDER_COLOR);
    draw_rectangle(x + 1.0, y + 1.0, w - 2.0, h - 2.0, UI_INNER_BORDER_COLOR);
    draw_rectangle(x + 2.0, y + 2.0, w - 4.0, h - 4.0, UI_BG_COLOR);
}
pub fn draw_square(x: i16, y: i16, w: i16, h: i16) {
    let x = x as f32;
    let y = y as f32;
    let w = w as f32;
    let h = h as f32;
    draw_rectangle(x, y, w, h, UI_OUTER_BORDER_COLOR);
    draw_rectangle(x + 1.0, y + 1.0, w - 2.0, h - 2.0, UI_BG_COLOR);
}
pub fn draw_text(x: i16, y: i16, text: &str) {
    let x = x as f32;
    let y = y as f32 + 5.0;
    macroquad::text::draw_text(text, x, y, 8.0, BLACK);
}
