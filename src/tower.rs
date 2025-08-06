use std::collections::VecDeque;

use crate::cards::{Card, CardFunction, CardType, FiringContext, Projectile};

// this is kind of dumb but i did it like this okay
pub fn get_towers() -> [Tower; 3] {
    let tower1 = Tower {
        sprite: 0,
        card_slots: vec![None; 6],
        shoot_delay: 0.32,
        recharge_speed: 0.50,
        ..Default::default()
    };
    let tower2 = Tower {
        sprite: 3,
        card_slots: vec![None; 3],
        shoot_delay: 0.12,
        recharge_speed: 0.07,
        ..Default::default()
    };
    let tower3 = Tower {
        sprite: 6,
        card_slots: vec![None; 11],
        shoot_delay: 0.65,
        recharge_speed: 0.65,
        ..Default::default()
    };
    let tower3 = Tower {
        sprite: 9,
        card_slots: vec![None; 8],
        shoot_delay: 0.25,
        recharge_speed: 0.65,
        ..Default::default()
    };
    [tower1, tower2, tower3]
}
#[derive(Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
impl Direction {
    pub fn to_vector(&self) -> (isize, isize) {
        match self {
            Direction::Down => (0, -1),
            Direction::Up => (0, 1),
            Direction::Right => (1, 0),
            Direction::Left => (-1, 0),
        }
    }
}
impl Default for Direction {
    fn default() -> Self {
        Self::Left
    }
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
    pub recharge_speed: f32,
    pub delay_counter: f32,
    pub direction: Direction,
}

fn draw_next(deck: &mut VecDeque<Card>) -> Vec<Card> {
    let mut cards = Vec::new();
    let mut current_draw = 1;
    while let Some(mut card) = deck.pop_front() {
        match card.ty {
            CardType::Modifier => {
                cards.push(card);
            }
            CardType::Multidraw(draw) => {
                cards.push(card);
                current_draw += draw;
            }
            CardType::Projectile(_) => {
                current_draw -= 1;

                // if card is trigger, draw one more time and set that as this card's payload
                if card.is_trigger {
                    let payload = draw_next(deck);
                    if let CardFunction::SummonProjectile(projectile) = &mut card.function {
                        projectile.payload = payload;
                    }
                }
                cards.push(card);

                if current_draw == 0 {
                    break;
                }
            }
        }
    }
    cards
}

fn apply_modifiers_to_context(context: &mut FiringContext, deck: &Vec<Card>) {
    for card in deck {
        match &card.function {
            CardFunction::ModifyContext(function) => function(context),
            CardFunction::SummonProjectile(projectile) => {
                if !projectile.payload.is_empty() {
                    // in noita, modifiers on spells in the payload affect the entire wand's recharge speed.
                    // this code is just to emulate that.
                    let mut mock_context = FiringContext::default();
                    apply_modifiers_to_context(&mut mock_context, &projectile.payload);
                    context.recharge_speed += mock_context.recharge_speed;
                }
            }
            _ => {}
        }
    }
}

pub fn fire_deck(
    origin_x: usize,
    origin_y: usize,
    direction: Direction,
    deck: Vec<Card>,
    context: &mut FiringContext,
) {
    apply_modifiers_to_context(context, &deck);
    for card in deck {
        match card.function {
            CardFunction::SummonProjectile(mut projectile) => {
                // apply context to projectile
                projectile.speed = (projectile.speed as isize + context.speed) as usize;
                projectile.lifetime = (projectile.lifetime as isize + context.lifetime) as usize;
                projectile.piercing |= context.piercing;

                for (damage_type, amt) in &context.damage_modifiers {
                    let old = projectile.damage.get(damage_type).unwrap().clone();
                    *projectile.damage.get_mut(damage_type).unwrap() =
                        (old as isize + amt) as usize;
                }
                // set projectile's starting pos and orientation
                projectile.x = origin_x;
                projectile.y = origin_y;
                projectile.direction = direction;
                context.spawn_list.push(projectile);
            }
            _ => {}
        }
    }
}

impl Tower {
    pub fn can_shoot(&self) -> bool {
        self.delay_counter <= 0.0
    }
    pub fn shoot(&mut self) -> Vec<Projectile> {
        let (drawn, should_recharge) = self.draw_next();
        let mut context = FiringContext::default();
        context.recharge_speed = self.recharge_speed;
        context.shoot_delay = self.shoot_delay;
        fire_deck(self.x, self.y, self.direction, drawn, &mut context);

        let mut cooldown = context.shoot_delay;
        if should_recharge {
            self.card_index = 0;
            cooldown = cooldown.max(context.recharge_speed)
        }
        self.delay_counter = cooldown;

        context.spawn_list
    }
    fn draw_next(&mut self) -> (Vec<Card>, bool) {
        let mut deck: VecDeque<Card> = self.card_slots.clone().into_iter().flatten().collect();
        for _ in 0..self.card_index {
            let popped = deck.pop_front().unwrap();
            deck.push_back(popped);
        }
        let old_length = deck.len();
        let drawn = draw_next(&mut deck);
        let new_length = deck.len();
        let amount_fired = old_length - new_length;
        self.card_index += amount_fired;
        (drawn, self.card_index >= old_length)
    }
}
