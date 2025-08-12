use std::{collections::HashMap, fmt::Debug};

use macroquad::{color::Color, math::Vec2, rand, shapes::draw_rectangle};

use crate::{
    assets::ProjectileSound, consts::*, map::Spritesheet, particle::Particle, tower::fire_deck,
};

pub mod library;

/// Returns all player-achievable cards
pub fn get_cards() -> Vec<Card> {
    let mut cards = vec![
        // modifiers
        library::aiming(),
        library::homing(),
        library::speed(),
        library::acidify(),
        library::piercing(),
        library::supercharge(),
        library::high_precision(),
        library::scatter(),
        library::ghost_shot(),
        library::shock(),
        library::freezeify(),
        library::boomerangify(),
        // multidraw
        library::double(),
        library::triple(),
        // projectiles
        library::rocket(),
        library::bomb(),
        library::magicbolt(),
        library::bubble(),
        library::explosion(),
        library::stun_explosion(),
        library::playing_card(),
        library::fireball(),
        library::thorn_dart(),
        library::dart(),
        library::acid_flask(),
        library::razor(),
        library::icecicle(),
        library::road_thorns(),
        library::banana(),
        library::death_ray(),
        library::sunbeam(),
        library::freeze_ray(),
        library::hammer(),
        library::lightning(),
        library::shotgun(),
        library::potato(),
        library::yoyo(),
    ];

    if cards.len() as u8 > u8::MAX / 2 {
        panic!("too many cards to represent as u8 in save data!");
    }

    let mut triggers = Vec::new();
    // generate trigger variants of projectile cards that allow it
    for card in &cards {
        if let CardType::Projectile(_, trigger_allowed) = card.ty {
            if trigger_allowed {
                triggers.push(library::as_trigger(card.clone()));
            }
        }
    }

    cards.append(&mut triggers);
    cards
}

fn sort_cards_to_tiers(cards: &[Card]) -> [Vec<&Card>; 4] {
    let mut tiers = std::array::from_fn(|_| Vec::new());
    for card in cards {
        tiers[card.tier as usize].push(card);
    }
    tiers
}

pub fn get_random_shop_card(round: usize, cards: &[Card]) -> Card {
    let round = round as u8;

    let mut tier = 0;
    let tiers = sort_cards_to_tiers(cards);

    // chance to increment tier. chance gets higher as rounds progress
    loop {
        if rand::gen_range(0, 100_u8).saturating_sub(round / 3) < UPGRADE_TIER_CHANCE {
            if tier >= 2 {
                break;
            }
            tier += 1;
        } else {
            break;
        }
    }

    let tier_cards = &tiers[tier as usize];
    tier_cards[rand::gen_range(0, tier_cards.len())].clone()
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
    pub x: f32,
    pub y: f32,
    pub spawn_x: f32,
    pub spawn_y: f32,
    /// How much larger hitbox the projectile has than usual
    pub extra_size: f32,
    /// Direction projectile is traveling
    pub direction: Vec2,
    /// Drag makes projectile slow down as it travels. 0.0 - 1.0, 1.0 makes it stop immediately after first frame
    pub drag: f32,
    pub draw_type: ProjectileDrawType,
    pub life: f32,
    /// Payload released on hit. Only used on trigger projectiles.
    pub payload: Vec<Card>,
    /// Released on death. Used by ex. the bomb exploding when its lifetime runs out.
    pub death_payload: Vec<Card>,
    /// Ghost frames allow projectile to travel through walls.
    /// Different to [CardModifierData]'s ghost, as this is only for a couple of frames.
    /// Used such that payloads of projectiles that hit a wall are allowed a couple frames
    /// to bounce away.
    pub ghost_frames: u8,
    /// How many "clones" the projectile has. Ex. the shotgun spell shoots 3 projectiles at a time,
    /// therefore it has 2 clones.
    pub clones_amount: u8,
    pub only_enemy_triggers: bool,
    /// Is the projectile immune to being rotated, i.e. by homing modifier?
    pub straight: bool,
    pub hit_sound: ProjectileSound,
    pub fire_sound: ProjectileSound,
    pub random_damage: Option<(u8, u8)>,
    pub modifier_data: CardModifierData,
}
impl Projectile {
    pub fn fire_payload(&self) -> Vec<Projectile> {
        let mut context = FiringContext::default();
        fire_deck(
            self.x,
            self.y,
            self.direction,
            self.payload.clone(),
            &mut context,
        );
        context.spawn_list
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DamageType {
    Magic,
    Pierce,
    Burn,
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
    pub spawn_list: Vec<Projectile>,
    pub modifier_data: CardModifierData,
}

#[derive(Clone, Default)]
pub struct CardModifierData {
    pub shoot_delay: f32,
    pub recharge_speed: f32,
    pub aim: bool,
    pub homing: bool,
    /// How many frames of stun does this give enemies?
    pub stuns: u8,
    pub lifetime: f32,
    /// Can projectile hit multiple enemies
    pub piercing: bool,
    /// Stops projectile from interacting with enemies at all, like the bomb,
    /// which doesn't get destroyed, nor deal damage on impact.
    pub anti_piercing: bool,
    /// Can projectile travel through walls/obstacles
    pub ghost: bool,
    /// Does projectile arc back towards caster
    pub boomerang: bool,
    pub speed: f32,
    /// Degrees (in radians) of spread/inaccuracy
    pub spread: f32,
    pub damage: HashMap<DamageType, f32>,
}
impl CardModifierData {
    pub fn iter(&self) -> Vec<(&'static str, String)> {
        let mut fields = Vec::new();
        for (k, v) in &self.damage {
            let k = match k {
                DamageType::Acid => "damage acid",
                DamageType::Cold => "damage cold",
                DamageType::Burn => "damage burn",
                DamageType::Magic => "damage magic",
                DamageType::Pierce => "damage pierce",
            };
            fields.push((k, v.to_string()));
        }
        for (k, field) in [
            ("shoot delay", self.shoot_delay),
            ("reload time", self.recharge_speed),
            ("speed", self.speed),
        ] {
            if fields.len() >= 3 {
                break;
            }
            if field != 0.0 {
                fields.push((k, field.to_string()));
            }
        }
        if fields.len() < 3 && self.spread != 0.0 {
            fields.push(("spread", self.spread.to_degrees().to_string() + " deg"));
        }

        fields
    }
    /// Like [CardModifierData::merge] but only merges shoot_delay, recharge_speed fields, which are the only fields that
    /// projectile type cards modify
    pub fn merge_projectile(&mut self, other: &CardModifierData) {
        self.shoot_delay += other.shoot_delay;
        self.recharge_speed += other.recharge_speed;
    }
    pub fn merge(&mut self, other: &CardModifierData) {
        self.shoot_delay += other.shoot_delay;
        self.recharge_speed += other.recharge_speed;
        self.aim |= other.aim;
        self.homing |= other.homing;
        self.lifetime += other.lifetime;
        self.piercing |= other.piercing;
        self.ghost |= other.ghost;
        self.boomerang |= other.boomerang;
        self.speed += other.speed;
        self.spread += other.spread;
        self.stuns += other.stuns;
        for (k, v) in &other.damage {
            if let Some(amt) = self.damage.get_mut(k) {
                *amt += v;
            } else {
                self.damage.insert(*k, *v);
            }
        }
    }
}

#[derive(Clone, Default)]
/// A card used by towers
pub struct Card {
    pub name: &'static str,
    pub desc: &'static str,
    pub tier: u8,
    pub ty: CardType,
    pub sprite: usize,
    pub is_trigger: bool,
}
impl Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Card({})", self.name))
    }
}
impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Card {
    pub fn draw(&self, card_sheet: &Spritesheet, x: f32, y: f32) {
        draw_rectangle(
            x - 2.0,
            y - 2.0,
            SPRITE_SIZE + 4.0,
            SPRITE_SIZE + 4.0,
            self.ty.get_border_color(),
        );
        draw_rectangle(
            x - 1.0,
            y - 1.0,
            SPRITE_SIZE + 2.0,
            SPRITE_SIZE + 2.0,
            COLOR_BEIGE,
        );
        card_sheet.draw_tile(x, y, self.sprite, false, 0.0);
        if self.is_trigger {
            card_sheet.draw_tile(x - 1.0, y - 1.0, 32 * 3, false, 0.0);
        }
    }
}
