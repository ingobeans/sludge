use crate::consts::*;

pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

/// A card used by towers
pub struct Card {}

/// A user placed tower
pub struct Tower {
    pub x: usize,
    pub y: usize,
    pub sprite: usize,
    pub cards: Vec<Card>,
    pub card_index: usize,
    pub shoot_delay: f32,
    pub delay_counter: f32,
    pub direction: Direction,
}
