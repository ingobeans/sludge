use std::f32::consts::PI;

use macroquad::prelude::*;

use crate::{consts::*, map::Spritesheet};

pub struct ParticleContext {
    pub x: f32,
    pub y: f32,
    pub origin_x: f32,
    pub origin_y: f32,
    pub direction: Vec2,
}
#[derive(Clone)]
pub struct Particle {
    pub life: u8,
    pub lifetime: u8,
    pub function: &'static dyn Fn(&Particle, &ParticleContext, &Spritesheet),
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
    ctx: &ParticleContext,
    particles: &Spritesheet,
    colors: (Color, Color),
    sprite: usize,
    size: f32,
) {
    if life <= 3 {
        let width = 8.0 * 3.0;
        let height = size;
        let mut params = DrawRectangleParams {
            offset: Vec2::ZERO,
            rotation: ctx.direction.to_angle(),
            color: colors.0,
        };
        draw_rectangle_ex(ctx.origin_x, ctx.origin_y, width, height, params.clone());
        params.color = colors.1;

        let offset_angle = ctx.direction.perp();
        let offset = Vec2::new(ctx.origin_x, ctx.origin_y) + offset_angle;
        draw_rectangle_ex(offset.x, offset.y, width, height - 2.0, params);
    }
    basic_animation_particle(
        life,
        lifetime,
        ctx.origin_x,
        ctx.origin_y - 2.0,
        sprite,
        1,
        3,
        particles,
    );
}

pub const NEW_TOWER: Particle = Particle {
    life: 0,
    lifetime: 60,
    function: &|this, ctx, _| {
        let a = (1.0 - this.life as f32 / 30 as f32).max(0.0);
        draw_circle_lines(
            ctx.x + SPRITE_SIZE / 2.0,
            ctx.y + SPRITE_SIZE / 2.0,
            SCREEN_WIDTH / 2.0 * a + SPRITE_SIZE,
            200.0,
            WHITE,
        );
    },
};

pub const YOYO: Particle = Particle {
    life: 0,
    lifetime: 0,
    function: &|this, ctx, particles| {
        draw_line(ctx.origin_x, ctx.origin_y, ctx.x, ctx.y, 1.0, COLOR_WHITE);
        particles.draw_tile(
            ctx.x - SPRITE_SIZE / 2.0,
            ctx.y - SPRITE_SIZE / 2.0,
            19,
            false,
            (15.0 - this.life as f32 % 30.0) / 15.0 * PI,
        );
    },
};

pub const SHOTGUN: Particle = Particle {
    life: 0,
    lifetime: 0,
    function: &|this, ctx, particles| {
        // draw little explosion at origin for the first couple of frames
        if this.life <= 3 {
            basic_animation_particle(
                this.life,
                3,
                ctx.origin_x,
                ctx.origin_y,
                32 + 16,
                1,
                3,
                particles,
            );
        }

        particles.draw_tile(
            ctx.x - SPRITE_SIZE / 2.0,
            ctx.y - SPRITE_SIZE / 2.0,
            17,
            false,
            ctx.direction.to_angle(),
        );
    },
};

pub const LIGHTNING: Particle = Particle {
    life: 0,
    lifetime: 5,
    function: &|this, ctx, particles| {
        ray_particle(
            this.life,
            this.lifetime,
            ctx,
            particles,
            (COLOR_CYAN, COLOR_CYAN),
            64 + 13,
            1.0,
        );
    },
};

pub const DEATH_RAY: Particle = Particle {
    life: 0,
    lifetime: 5,
    function: &|this, ctx, particles| {
        ray_particle(
            this.life,
            this.lifetime,
            ctx,
            particles,
            (COLOR_RED, COLOR_BLACK),
            32 + 10,
            4.0,
        );
    },
};
pub const SUNBEAM: Particle = Particle {
    life: 0,
    lifetime: 5,
    function: &|this, ctx, particles| {
        ray_particle(
            this.life,
            this.lifetime,
            ctx,
            particles,
            (COLOR_YELLOW, COLOR_WHITE),
            32 * 2 + 10,
            4.0,
        );
    },
};
pub const FREEZE_RAY: Particle = Particle {
    life: 0,
    lifetime: 5,
    function: &|this, ctx, particles| {
        ray_particle(
            this.life,
            this.lifetime,
            ctx,
            particles,
            (COLOR_CYAN, COLOR_WHITE),
            32 * 3 + 10,
            4.0,
        );
    },
};

pub const HIT_MARKER: Particle = Particle {
    life: 0,
    lifetime: 2,
    function: &|_, ctx, particles| {
        particles.draw_tile(
            ctx.x - SPRITE_SIZE / 2.0,
            ctx.y - SPRITE_SIZE / 2.0,
            6,
            false,
            0.0,
        );
    },
};

pub const BUBBLE: Particle = Particle {
    life: 0,
    lifetime: 19,
    function: &|this, ctx, particles| {
        let stage_1_end = 10;
        let frame_amt = 3;
        let anim_frame_offset = (this.life.saturating_sub(stage_1_end) / (10 / frame_amt)) as usize;

        let move_y = this.life.min(stage_1_end) * 2 / stage_1_end;

        particles.draw_tile(
            ctx.x - SPRITE_SIZE / 2.0,
            ctx.y - move_y as f32 - SPRITE_SIZE / 2.0,
            3 + anim_frame_offset,
            false,
            0.0,
        );
    },
};

pub const EXPLOSION: Particle = Particle {
    life: 0,
    lifetime: 10,
    function: &|this, ctx, particles| {
        basic_animation_particle(this.life, this.lifetime, ctx.x, ctx.y, 32, 2, 3, particles);
    },
};

pub const STAR_EXPLOSION: Particle = Particle {
    life: 0,
    lifetime: 15,
    function: &|this, ctx, particles| {
        basic_animation_particle(
            this.life,
            this.lifetime,
            ctx.x,
            ctx.y,
            1 * 32 + 20,
            2,
            5,
            particles,
        );
    },
};
pub const FIRE_EXPLOSION: Particle = Particle {
    life: 0,
    lifetime: 10,
    function: &|this, ctx, particles| {
        basic_animation_particle(
            this.life,
            this.lifetime,
            ctx.x,
            ctx.y,
            5 * 32,
            2,
            3,
            particles,
        );
    },
};

pub const STUN_EXPLOSION: Particle = Particle {
    life: 0,
    lifetime: 5,
    function: &|this, ctx, particles| {
        basic_animation_particle(
            this.life,
            this.lifetime,
            ctx.x,
            ctx.y,
            5 * 32 + 6,
            2,
            3,
            particles,
        );
    },
};

pub const FIREBALL: Particle = Particle {
    life: 0,
    lifetime: 10,
    function: &|this, ctx, particles| {
        let frame_amt = 3;
        let anim_frame_offset = (this.life as usize) / (10 / frame_amt) % frame_amt;

        particles.draw_tile(
            ctx.x - SPRITE_SIZE / 2.0,
            ctx.y - SPRITE_SIZE / 2.0,
            38 + anim_frame_offset,
            false,
            ctx.direction.to_angle(),
        );
    },
};

pub const ACID_PUDDLE: Particle = Particle {
    life: 0,
    lifetime: 10,
    function: &|this, ctx, particles| {
        basic_animation_particle(
            this.life,
            this.lifetime,
            ctx.x,
            ctx.y,
            7 * 32,
            2,
            2,
            particles,
        );
    },
};

pub const CONFETTIS: [Particle; 4] = [
    Particle {
        life: 0,
        lifetime: 10,
        function: &|this, ctx, particles| {
            basic_animation_particle(this.life, 10, ctx.x, ctx.y, 32 * 3 + 13, 1, 4, particles);
        },
    },
    Particle {
        life: 0,
        lifetime: 10,
        function: &|this, ctx, particles| {
            basic_animation_particle(this.life, 10, ctx.x, ctx.y, 32 * 4 + 13, 1, 4, particles);
        },
    },
    Particle {
        life: 0,
        lifetime: 10,
        function: &|this, ctx, particles| {
            basic_animation_particle(this.life, 10, ctx.x, ctx.y, 32 * 5 + 13, 1, 4, particles);
        },
    },
    Particle {
        life: 0,
        lifetime: 10,
        function: &|this, ctx, particles| {
            basic_animation_particle(this.life, 10, ctx.x, ctx.y, 32 * 6 + 13, 1, 4, particles);
        },
    },
];
