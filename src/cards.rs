use std::collections::HashMap;

use crate::{particle::Particle, tower::Direction};

mod library;

pub fn get_cards() -> [Card; 3] {
    [library::aiming(), library::magicbolt(), library::bomb()]
}

#[derive(Clone, Copy)]
pub enum CardType {
    /// bool is whether projectile is allowed to be a trigger
    Projectile(bool),
    Modifier,
    /// usize is how many cards to draw.
    Multidraw(usize),
}
impl Default for CardType {
    fn default() -> Self {
        Self::Projectile(false)
    }
}

#[derive(Clone)]
pub enum ProjectileDrawType {
    Sprite(usize),
    Particle(Particle),
    None,
}
impl Default for ProjectileDrawType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default, Clone)]
pub struct Projectile {
    pub x: usize,
    pub y: usize,
    pub direction: Direction,
    pub draw_type: ProjectileDrawType,
    pub speed: usize,
    pub lifetime: usize,
    pub life: usize,
    pub piercing: bool,
    pub damage: HashMap<DamageType, usize>,
    pub payload: Vec<Card>,
    pub inate_payload: Vec<Card>,
    pub death_payload: Vec<Card>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum DamageType {
    Magic,
    Pierce,
    Explosion,
    Cold,
    Acid,
}
impl Default for DamageType {
    fn default() -> Self {
        Self::Magic
    }
}
#[derive(Clone, Default)]
pub struct FiringContext {
    pub shoot_delay: f32,
    pub recharge_speed: f32,
    pub draw_count: usize,
    pub aim: bool,
    pub damage_modifiers: HashMap<DamageType, isize>,
    pub lifetime: isize,
    pub piercing: bool,
    pub speed: isize,
    pub spawn_list: Vec<Projectile>,
    pub origin_x: usize,
    pub origin_y: usize,
}

#[derive(Clone)]
pub enum CardFunction {
    SummonProjectile(Projectile),
    ModifyContext(&'static dyn Fn(&mut FiringContext)),
    None,
}
impl Default for CardFunction {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Default)]
/// A card used by towers
pub struct Card {
    pub ty: CardType,
    pub sprite: usize,
    pub function: CardFunction,
    pub is_trigger: bool,
}
