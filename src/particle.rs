use macroquad::prelude::*;

use crate::{consts::*, map::Spritesheet};

#[derive(Clone)]
pub struct Particle {
    pub life: u8,
    pub lifetime: u8,
    pub function: &'static dyn Fn(&Particle, f32, f32, &Vec2, &Spritesheet),
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
    let anim_frame_offset =
        (life as f32 / (lifetime as f32 / frame_amt as f32)) as usize % frame_amt * size;
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
fn ray_particle(
    life: u8,
    lifetime: u8,
    x: f32,
    y: f32,
    direction: &Vec2,
    particles: &Spritesheet,
    colors: (Color, Color),
    sprite: usize,
) {
    let projectile_speed = 8.0;
    let travel_frames = life.min(3);
    let origin_x = x - direction.x * projectile_speed * travel_frames as f32;
    let origin_y = y - direction.y * projectile_speed * travel_frames as f32 + 4.0;

    if life <= 3 {
        let width = 8.0 * 3.0;
        let height = 4.0;
        let mut params = DrawRectangleParams {
            offset: Vec2::ZERO,
            rotation: direction.to_angle(),
            color: colors.0,
        };
        draw_rectangle_ex(origin_x, origin_y, width, height, params.clone());
        params.color = colors.1;
        draw_rectangle_ex(origin_x, origin_y - 1.0, width, height - 2.0, params);
    }
    basic_animation_particle(
        life,
        lifetime,
        origin_x,
        origin_y - 2.0,
        sprite,
        1,
        3,
        particles,
    );
}

pub const STUNNED: Particle = Particle {
    life: 0,
    lifetime: 20,
    function: &|this, x, y, _, particles| {
        basic_animation_particle(this.life, 10, x + 4.0, y, 32 + 13, 1, 3, particles);
    },
};

pub const DEATH_RAY: Particle = Particle {
    life: 0,
    lifetime: 5,
    function: &|this, x, y, direction, particles| {
        ray_particle(
            this.life,
            this.lifetime,
            x,
            y,
            direction,
            particles,
            (COLOR_RED, COLOR_BLACK),
            32 + 10,
        );
    },
};
pub const SUNBEAM: Particle = Particle {
    life: 0,
    lifetime: 5,
    function: &|this, x, y, direction, particles| {
        ray_particle(
            this.life,
            this.lifetime,
            x,
            y,
            direction,
            particles,
            (COLOR_YELLOW, COLOR_WHITE),
            32 * 2 + 10,
        );
    },
};
pub const FREEZE_RAY: Particle = Particle {
    life: 0,
    lifetime: 5,
    function: &|this, x, y, direction, particles| {
        ray_particle(
            this.life,
            this.lifetime,
            x,
            y,
            direction,
            particles,
            (COLOR_CYAN, COLOR_WHITE),
            32 * 3 + 10,
        );
    },
};

pub const HIT_MARKER: Particle = Particle {
    life: 0,
    lifetime: 2,
    function: &|_, x, y, _, particles| {
        particles.draw_tile(x, y, 6, false, 0.0);
    },
};

pub const BUBBLE: Particle = Particle {
    life: 0,
    lifetime: 19,
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

pub const STUN_EXPLOSION: Particle = Particle {
    life: 0,
    lifetime: 5,
    function: &|this, x, y, _, particles| {
        basic_animation_particle(this.life, this.lifetime, x, y, 5 * 32 + 6, 2, 3, particles);
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
