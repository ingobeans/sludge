use macroquad::math::Vec2;

use crate::{consts::*, map::Spritesheet};

#[derive(Clone)]
pub struct Particle {
    pub life: u8,
    pub lifetime: u8,
    pub function: &'static dyn Fn(&Particle, f32, f32, Vec2, &Spritesheet),
}

fn basic_animation_particle(
    life: u8,
    lifetime: u8,
    x: f32,
    y: f32,
    sprite: usize,
    size: usize,
    frame_amt: usize,
    particles: &Spritesheet,
) {
    let anim_frame_offset = (life as usize / (lifetime as usize / frame_amt)) % frame_amt * size;
    for i in 0..size {
        for j in 0..size {
            particles.draw_tile(
                (x + j as f32 * SPRITE_SIZE) - (size as f32 * SPRITE_SIZE / 2.0),
                (y + i as f32 * SPRITE_SIZE) - (size as f32 * SPRITE_SIZE / 2.0),
                sprite + anim_frame_offset + i * 32 + j,
                false,
                0.0,
            );
        }
    }
}

pub const HIT_MARKER: Particle = Particle {
    life: 0,
    lifetime: 2,
    function: &|_, x, y, _, particles| {
        particles.draw_tile(x, y, 6, false, 0.0);
    },
};

pub const BUBBLE: Particle = Particle {
    life: 0,
    lifetime: 20,
    function: &|this, x, y, _, particles| {
        let stage_1_end = 10;
        let frame_amt = 3;
        let anim_frame_offset = (this.life.saturating_sub(stage_1_end) / (10 / frame_amt)) as usize;

        let move_y = this.life.min(stage_1_end) * 2 / stage_1_end;

        particles.draw_tile(x, y - move_y as f32, 3 + anim_frame_offset, false, 0.0);
    },
};

pub const EXPLOSION: Particle = Particle {
    life: 0,
    lifetime: 10,
    function: &|this, x, y, _, particles| {
        basic_animation_particle(this.life, this.lifetime, x, y, 32, 2, 3, particles);
    },
};

pub const FIRE_EXPLOSION: Particle = Particle {
    life: 0,
    lifetime: 10,
    function: &|this, x, y, _, particles| {
        basic_animation_particle(this.life, this.lifetime, x, y, 5 * 32, 2, 3, particles);
    },
};

pub const FIREBALL: Particle = Particle {
    life: 0,
    lifetime: 10,
    function: &|this, x, y, direction, particles| {
        let frame_amt = 3;
        let anim_frame_offset = (this.life as usize) / (10 / frame_amt) % frame_amt;

        particles.draw_tile(x, y, 38 + anim_frame_offset, false, direction.to_angle());
    },
};

pub const ACID_PUDDLE: Particle = Particle {
    life: 0,
    lifetime: 10,
    function: &|this, x, y, _, particles| {
        basic_animation_particle(this.life, 10, x, y, 7 * 32, 2, 2, particles);
    },
};
