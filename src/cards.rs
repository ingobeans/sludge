use std::collections::HashMap;

use macroquad::math::Vec2;

use crate::particle::Particle;

mod library;

pub fn get_cards() -> [Card; 4] {
    [
        library::aiming(),
        library::magicbolt(),
        library::bomb(),
        library::speed(),
    ]
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
pub enum SpriteRotationMode {
    /// Fixed/no rotation
    None,
    /// Faces direction it is traveling
    Direction,
    /// It goes round and round
    Spin,
}

#[derive(Clone)]
pub enum ProjectileDrawType {
    Sprite(usize, SpriteRotationMode),
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
    /// Direction projectile is traveling
    pub direction: Vec2,
    pub draw_type: ProjectileDrawType,
    pub life: isize,
    /// Payload released on hit. Only used on trigger projectiles.
    pub payload: Vec<Card>,
    /// Like payload, except inate to the projectile, like the rocket releasing explosion on hit.
    /// The reason for the seperation is so that modifiers from the main payload don't affect the inate payload.
    pub inate_payload: Vec<Card>,
    /// Released on death. Used by ex. the bomb exploding when its lifetime runs out.
    pub death_payload: Vec<Card>,
    pub modifier_data: CardModifierData,
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
    pub draw_count: usize,
    pub damage_modifiers: HashMap<DamageType, isize>,
    pub spawn_list: Vec<Projectile>,
    pub origin_x: usize,
    pub origin_y: usize,
    pub modifier_data: CardModifierData,
}

#[derive(Clone, Default)]
pub struct CardModifierData {
    pub shoot_delay: f32,
    pub recharge_speed: f32,
    pub aim: bool,
    pub lifetime: isize,
    pub piercing: bool,
    pub speed: isize,
    pub damage: HashMap<DamageType, usize>,
}
impl CardModifierData {
    /// Like [CardModifierData::merge] but only merges shoot_delay and recharge_speed fields, which are the only fields that
    /// projectile type cards modify
    pub fn merge_projectile(&mut self, other: &CardModifierData) {
        self.shoot_delay += other.shoot_delay;
        self.recharge_speed += other.recharge_speed;
    }
    pub fn merge(&mut self, other: &CardModifierData) {
        self.shoot_delay += other.shoot_delay;
        self.recharge_speed += other.recharge_speed;
        self.aim |= other.aim;
        self.lifetime += other.lifetime;
        self.piercing |= other.piercing;
        self.speed += other.speed;
        for (k, v) in &other.damage {
            if let Some(amt) = self.damage.get_mut(&k) {
                *amt += v;
            } else {
                self.damage.insert(k.clone(), *v);
            }
        }
    }
}

#[derive(Clone, Default)]
/// A card used by towers
pub struct Card {
    pub ty: CardType,
    pub sprite: usize,
    pub is_trigger: bool,
    pub modifier_data: CardModifierData,
    pub projectile: Option<Projectile>,
}
