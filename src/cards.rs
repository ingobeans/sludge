use crate::consts::*;
use std::sync::LazyLock;

// this is kind of dumb but i did it like this okay
pub static TOWERS: LazyLock<[Tower; 3]> = LazyLock::new(|| {
    let tower1 = Tower {
        sprite: 0,
        card_slots: vec![None; 6],
        shoot_delay: 0.32,
        ..Default::default()
    };
    let tower2 = Tower {
        sprite: 3,
        card_slots: vec![None; 3],
        shoot_delay: 0.12,
        ..Default::default()
    };
    let tower3 = Tower {
        sprite: 6,
        card_slots: vec![None; 11],
        shoot_delay: 0.65,
        ..Default::default()
    };
    [tower1, tower2, tower3]
});

#[derive(Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
impl Default for Direction {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Clone, Copy)]
/// A card used by towers
pub struct Card {
    pub sprite: usize,
}

#[derive(Clone, Default)]
/// A user placed tower
pub struct Tower {
    pub x: usize,
    pub y: usize,
    pub sprite: usize,
    pub card_slots: Vec<Option<Card>>,
    pub card_index: usize,
    pub shoot_delay: f32,
    pub delay_counter: f32,
    pub direction: Direction,
}
