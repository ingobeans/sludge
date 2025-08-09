use macroquad::prelude::*;

pub const SCREEN_WIDTH: f32 = 192.0;
pub const SCREEN_HEIGHT: f32 = 144.0;
pub const SPRITE_SIZE: f32 = 8.0;
pub const CARD_SIZE: f32 = SPRITE_SIZE + 4.0;

pub const INV_SLOTS_HORIZONTAL: usize = 2;
pub const INV_SLOTS_VERTICAL: usize = (SCREEN_HEIGHT_USIZE - 4) / CARD_SIZE_USIZE;

pub const SHOP_SLOTS_HORIZONTAL: usize = 5;

pub const SHOP_WIDTH: f32 = SHOP_SLOTS_HORIZONTAL as f32 * SHOP_CARD_SIZE + 4.0;
pub const SHOP_PADDING: f32 = 24.0;
pub const SHOP_HEIGHT: f32 = CARD_SIZE + 4.0 + SHOP_PADDING;
pub const SHOP_CARD_SIZE: f32 = CARD_SIZE + 4.0;

pub const INV_WIDTH: f32 = INV_SLOTS_HORIZONTAL as f32 * CARD_SIZE + 4.0;

pub const LEFT: Vec2 = Vec2::new(-1.0, 0.0);
pub const SPREAD: f32 = 4.0_f32.to_radians();
pub const DEFAULT_SPAWN_DELAY: u8 = 24;

pub const STARTING_LIVES: u8 = 40;
pub const STARTING_GOLD: u16 = 100;
pub const GOLD_ROUND_REWARD: u16 = 45;

pub const FREEZE_TIME: u8 = 90;

pub const CARD_INFO_WIDTH: f32 = 64.0 + 32.0;
pub const CARD_INFO_HEIGHT: f32 = 32.0;

pub const UI_BG_COLOR: Color = Color::from_hex(0xb86f50);
pub const UI_BORDER_COLOR: Color = Color::from_hex(0x743f39);
pub const UI_BUTTON_BG_COLOR: Color = Color::from_hex(0xffe762);

// define usize variants
pub const SCREEN_WIDTH_USIZE: usize = SCREEN_WIDTH as usize;
pub const SCREEN_HEIGHT_USIZE: usize = SCREEN_HEIGHT as usize;
pub const SPRITE_SIZE_USIZE: usize = SPRITE_SIZE as usize;
pub const INV_WIDTH_USIZE: usize = INV_WIDTH as usize;
pub const CARD_SIZE_USIZE: usize = CARD_SIZE as usize;
