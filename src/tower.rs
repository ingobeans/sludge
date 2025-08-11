use std::collections::VecDeque;

use macroquad::{math::Vec2, rand::RandomRange};

use crate::{
    cards::{Card, CardType, FiringContext, Projectile},
    consts::*,
};

// this is kind of dumb but i did it like this okay
pub fn get_towers(spawnpoints: [(usize, usize); 4]) -> [Tower; 4] {
    let default = Tower {
        direction: LEFT,
        ..Default::default()
    };
    let tower1 = Tower {
        x: spawnpoints[0].0 as f32,
        y: spawnpoints[0].1 as f32,
        sprite: 0,
        card_slots: vec![None; 6],
        shoot_delay: 0.1,
        recharge_speed: 0.50,
        ..default
    };
    let tower2 = Tower {
        x: spawnpoints[1].0 as f32,
        y: spawnpoints[1].1 as f32,
        sprite: 3,
        card_slots: vec![None; 3],
        shoot_delay: 0.12,
        recharge_speed: 0.07,
        ..default
    };
    let tower3 = Tower {
        x: spawnpoints[2].0 as f32,
        y: spawnpoints[2].1 as f32,
        sprite: 6,
        card_slots: vec![None; 11],
        shoot_delay: 0.65,
        recharge_speed: 0.65,
        ..default
    };
    let tower4 = Tower {
        x: spawnpoints[3].0 as f32,
        y: spawnpoints[3].1 as f32,
        sprite: 9,
        card_slots: vec![None; 8],
        shoot_delay: 0.25,
        recharge_speed: 0.65,
        ..default
    };
    [tower1, tower2, tower3, tower4]
}

#[derive(Clone, Default)]
/// A user placed tower
pub struct Tower {
    pub x: f32,
    pub y: f32,
    pub sprite: usize,
    pub card_slots: Vec<Option<Card>>,
    pub card_index: usize,
    pub shoot_delay: f32,
    pub recharge_speed: f32,
    pub delay_counter: f32,
    pub direction: Vec2,
}
impl PartialEq for Tower {
    fn eq(&self, other: &Self) -> bool {
        self.sprite == other.sprite
    }
}

fn draw_next(deck: &mut VecDeque<Card>) -> Vec<Card> {
    let mut cards = Vec::new();
    let mut current_draw = 1;
    while let Some(mut card) = deck.pop_front() {
        match &mut card.ty {
            CardType::Modifier(_) => {
                cards.push(card);
            }
            CardType::Multidraw(draw) => {
                current_draw += *draw;
                current_draw -= 1;
                cards.push(card);
            }
            CardType::Projectile(projectile, _) => {
                current_draw -= 1;

                // if card is trigger, draw one more time and set that as this card's payload
                if card.is_trigger {
                    let payload = draw_next(deck);
                    projectile.payload = payload;
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
        match &card.ty {
            CardType::Modifier(modifier_data) => {
                context.modifier_data.merge(modifier_data);
            }
            CardType::Projectile(projectile, _) => {
                context
                    .modifier_data
                    .merge_projectile(&projectile.modifier_data);

                // in noita, modifiers on spells in the payload affect the entire wand's recharge speed.
                // the following code is just to emulate that.
                if !projectile.payload.is_empty() {
                    let mut mock_context = FiringContext::default();
                    apply_modifiers_to_context(&mut mock_context, &projectile.payload);
                    context.modifier_data.recharge_speed +=
                        mock_context.modifier_data.recharge_speed;
                }
            }
            _ => {}
        }
    }
}

pub fn fire_deck(
    origin_x: f32,
    origin_y: f32,
    direction: Vec2,
    deck: Vec<Card>,
    context: &mut FiringContext,
) {
    apply_modifiers_to_context(context, &deck);
    for card in deck {
        if let CardType::Projectile(mut projectile, _) = card.ty {
            projectile.modifier_data.merge(&context.modifier_data);

            let spread = RandomRange::gen_range(-SPREAD, SPREAD);
            projectile.x = origin_x;
            projectile.y = origin_y;
            projectile.direction = Vec2::from_angle(direction.to_angle() + spread);
            context.spawn_list.push(projectile);
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
        context.modifier_data.recharge_speed = self.recharge_speed;
        context.modifier_data.shoot_delay = self.shoot_delay;
        fire_deck(self.x, self.y, self.direction, drawn, &mut context);

        let mut cooldown = context.modifier_data.shoot_delay;
        self.delay_counter = context.modifier_data.shoot_delay;
        if should_recharge {
            self.card_index = 0;
            cooldown = cooldown.max(context.modifier_data.recharge_speed)
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
