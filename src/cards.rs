use std::collections::HashMap;

use macroquad::{color::Color, math::Vec2, shapes::draw_rectangle};

use crate::{consts::*, map::Spritesheet, particle::Particle};

mod library;

/// Returns all player-achievable cards
pub fn get_cards() -> Vec<Card> {
    let mut cards = vec![
        // modifiers
        library::aiming(),
        library::speed(),
        library::acidify(),
        // multidraw
        library::double(),
        library::triple(),
        // projectiles
        library::rocket(),
        library::bomb(),
        library::magicbolt(),
        library::bubble(),
    ];

    let mut triggers = Vec::new();
    // generate trigger variants of projectile cards that allow it
    for card in &cards {
        if let CardType::Projectile(_, trigger_allowed) = card.ty {
            if trigger_allowed {
                let mut trigger = card.clone();
                trigger.is_trigger = true;
                triggers.push(trigger);
            }
        }
    }

    cards.append(&mut triggers);
    cards
}

#[derive(Clone)]
pub enum CardType {
    /// bool is whether projectile is allowed to be a trigger
    Projectile(Projectile, bool),
    Modifier(CardModifierData),
    /// usize is how many cards to draw.
    Multidraw(usize),
}
impl CardType {
    fn get_border_color(&self) -> Color {
        match self {
            Self::Projectile(_, _) => Color::from_hex(0x9e2835),
            Self::Modifier(_) => Color::from_hex(0x4f6781),
            Self::Multidraw(_) => Color::from_hex(0xafbfd2),
        }
    }
}
impl Default for CardType {
    fn default() -> Self {
        Self::Modifier(CardModifierData::default())
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
    pub x: i16,
    pub y: i16,
    /// Direction projectile is traveling
    pub direction: Vec2,
    pub draw_type: ProjectileDrawType,
    pub life: i16,
    /// Payload released on hit. Only used on trigger projectiles.
    pub payload: Vec<Card>,
    /// Like payload, except inate to the projectile, like the rocket releasing explosion on hit.
    /// The reason for the seperation is so that modifiers from the main payload don't affect the inate payload.
    pub inate_payload: Vec<Card>,
    /// Released on death. Used by ex. the bomb exploding when its lifetime runs out.
    pub death_payload: Vec<Card>,
    pub modifier_data: CardModifierData,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
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
    pub damage_modifiers: HashMap<DamageType, i16>,
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
    pub lifetime: i16,
    pub piercing: bool,
    pub speed: i16,
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
            if let Some(amt) = self.damage.get_mut(k) {
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
}
impl Card {
    pub fn draw(&self, card_sheet: &Spritesheet, x: i16, y: i16) {
        draw_rectangle(
            x.saturating_sub(2) as f32,
            y.saturating_sub(2) as f32,
            (SPRITE_SIZE + 4) as f32,
            (SPRITE_SIZE + 4) as f32,
            self.ty.get_border_color(),
        );
        draw_rectangle(
            x.saturating_sub(1) as f32,
            y.saturating_sub(1) as f32,
            (SPRITE_SIZE + 2) as f32,
            (SPRITE_SIZE + 2) as f32,
            UI_BG_COLOR,
        );
        //draw_rectangle(
        //    x as f32,
        //    y as f32,
        //    SPRITE_SIZE as f32,
        //    SPRITE_SIZE as f32,
        //    UI_BG_COLOR,
        //);
        card_sheet.draw_tile(x, y, self.sprite, false, 0.0);
        if self.is_trigger {
            card_sheet.draw_tile(x, y, 32, false, 0.0);
        }
    }
}
