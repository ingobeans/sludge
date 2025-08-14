#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]

use std::f32::consts::PI;
use std::time::Instant;

use crate::assets::*;
use crate::cards::*;
use crate::consts::*;
use crate::enemy::*;
use crate::map::*;
use crate::particle::Particle;
use crate::particle::ParticleContext;
use crate::rounds::*;
use crate::save::*;
use crate::tower::*;
use crate::ui::*;
use macroquad::rand;
use macroquad::{miniquad::window::screen_size, prelude::*};

mod assets;
mod cards;
mod consts;
mod enemy;
mod map;
mod particle;
mod rounds;
mod save;
mod tower;
mod ui;

fn get_seed() -> u64 {
    macroquad::miniquad::date::now() as u64
}

fn get_direction_nearest_enemy(enemies: &Vec<Enemy>, x: f32, y: f32) -> Option<Vec2> {
    if enemies.is_empty() {
        return None;
    }
    let mut nearest: Option<(f32, Vec2)> = None;
    for enemy in enemies {
        if enemy.health <= 0.0 {
            continue;
        }
        let (enemy_x, enemy_y) = enemy.get_centre();
        let distance = ((enemy_x - x).powi(2) + (enemy_y - y).powi(2)).sqrt();
        if nearest.is_none() || distance < nearest.unwrap().0 {
            nearest = Some((distance, Vec2::new(enemy_x - x, enemy_y - y).normalize()));
        }
    }
    nearest.map(|f| f.1)
}
enum GameState {
    Running,
    Win,
    Lose,
    Paused,
}
struct Sludge {
    state: GameState,
    /// Just used to ensure same sublevels are used on saves
    seed: u64,
    map: Map,
    map_index: usize,
    enemies: Vec<Enemy>,
    towers: Vec<Tower>,
    projectiles: Vec<Projectile>,
    projectile_spawnlist: Vec<Projectile>,
    orphaned_particles: Vec<(Particle, ParticleContext)>,
    lives: u8,
    ui_manager: UIManager,
    round_manager: RoundManager,
    sfx_manager: SFXManager,
    just_selected_tower: bool,
    lab: bool,
    rotating_tower: bool,
    moving: Option<Tower>,
    selected: Option<usize>,
    tileset: Spritesheet,
    icon_sheet: Spritesheet,
    card_sheet: Spritesheet,
    particle_sheet: Spritesheet,
}
impl Sludge {
    async fn new(
        map: Map,
        map_index: usize,
        text_engine: TextEngine,
        lab: bool,
        seed: u64,
    ) -> Self {
        let tileset = load_spritesheet("data/assets/tileset.png", SPRITE_SIZE_USIZE);
        let icon_sheet = load_spritesheet("data/assets/entities.png", SPRITE_SIZE_USIZE);
        let card_sheet = load_spritesheet("data/assets/cards.png", SPRITE_SIZE_USIZE);
        let particle_sheet = load_spritesheet("data/assets/particles.png", SPRITE_SIZE_USIZE);

        // add starting towers
        let base_towers = get_towers(map.tower_spawnpoints);
        let towers = if !lab {
            base_towers[..2].into()
        } else {
            base_towers.into()
        };
        let round_manager = load_round_data(seed);
        let sfx_manager = SFXManager::new().await;

        Self {
            state: GameState::Running,
            seed,
            map,
            map_index,
            enemies: Vec::with_capacity(100),
            towers,
            projectiles: Vec::with_capacity(100),
            projectile_spawnlist: Vec::with_capacity(100),
            orphaned_particles: Vec::with_capacity(100),
            lives: STARTING_LIVES,
            round_manager,
            just_selected_tower: false,
            lab,
            rotating_tower: false,
            moving: None,
            selected: None,
            ui_manager: UIManager::new(text_engine),
            sfx_manager,
            tileset,
            icon_sheet,
            card_sheet,
            particle_sheet,
        }
    }
    fn start_round(&mut self) {
        self.projectiles.clear();
        if !self.lab {
            self.ui_manager.shop = None;
        }

        self.round_manager.in_progress = true;
    }
    fn spawn_enemy(&mut self, ty: &'static EnemyType) {
        let spawn = self.map.points[0];
        let enemy = Enemy::new(
            ty,
            spawn.0 * SPRITE_SIZE,
            spawn.1 * SPRITE_SIZE,
            EnemyState::default(),
        );
        self.enemies.push(enemy);
    }
    fn is_valid_tower_placement(&self, x: f32, y: f32) -> bool {
        for tower in &self.towers {
            let distance = ((tower.x - x).powi(2) + (tower.y - y).powi(2)).sqrt();
            if distance < SPRITE_SIZE {
                return false;
            }
        }
        self.map.is_unobstructed(x as usize, y as usize)
    }
    fn get_tower_near(&self, local_x: f32, local_y: f32) -> Option<(usize, f32)> {
        let mut clicked = None;
        for (index, tower) in self.towers.iter().enumerate() {
            let distance = ((tower.x + SPRITE_SIZE / 2.0 - local_x).powi(2)
                + (tower.y + SPRITE_SIZE / 2.0 - local_y).powi(2))
            .sqrt();
            if distance <= SPRITE_SIZE {
                if clicked.is_none() {
                    clicked = Some((index, distance))
                } else {
                    let old_distance = clicked.unwrap().1;
                    if distance < old_distance {
                        clicked = Some((index, distance))
                    }
                }
            }
        }
        clicked
    }
    fn handle_input(&mut self, local_x: f32, local_y: f32) {
        // if we're currently dragging a tower
        if self.moving.is_some() {
            let mut tower_x = 0.0;
            let mut tower_y = 0.0;
            if let Some(tower) = &mut self.moving {
                // set tower x and y to cursor position (offset by 4 so its centered, and clamped to be inside map)
                tower.x =
                    (local_x - SPRITE_SIZE / 2.0).clamp(0.0, SCREEN_WIDTH - 1.0 - SPRITE_SIZE);
                tower.y = local_y.min(SCREEN_HEIGHT - 1.0 - SPRITE_SIZE);

                // store new pos so we can access it outside of this `if let` scope
                tower_x = tower.x;
                tower_y = tower.y;
            }
            let valid = self.is_valid_tower_placement(tower_x, tower_y);
            // if we're no longer holding LMB and the spot is valid, place tower there
            if !is_mouse_button_down(MouseButton::Left) && valid {
                self.towers.push(self.moving.take().unwrap());
                self.selected = Some(self.towers.len() - 1);
            }
            // stop any other input events from being handled
            return;
        }

        if let Some(tower) = self.selected.map(|f| &mut self.towers[f]) {
            let x1 = (tower.x + SPRITE_SIZE / 2.0).round();
            let y1 = (tower.y + SPRITE_SIZE / 2.0).round();
            let x2 = (x1 + tower.direction.x * 16.0).round();
            let y2 = (y1 + tower.direction.y * 16.0).round();

            // if we're currently rotating a tower
            if self.rotating_tower {
                tower.direction = Vec2::new(local_x - x1, local_y - y1).normalize();
                if !is_mouse_button_down(MouseButton::Left)
                    && !is_mouse_button_down(MouseButton::Right)
                {
                    self.rotating_tower = false;
                }
                return;
            }

            if is_mouse_button_pressed(MouseButton::Right)
                || is_mouse_button_pressed(MouseButton::Left) && {
                    let distance = ((x2 - local_x).powi(2) + (y2 - local_y).powi(2)).sqrt();
                    distance < 2.0
                }
            {
                self.rotating_tower = true;
                return;
            }
        }

        // handle ui elements. if an element is interacted with, also return to stop other inputs being handled.
        let slots_amt = self
            .selected
            .map(|index| self.towers[index].card_slots.len())
            .unwrap_or(0);
        if self.ui_manager.is_ui_hovered(local_x, local_y, slots_amt) {
            return;
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            // find tower being clicked by iterating through all towers and checking if distance to cursor <= 8.0
            // (also sorted based on distance for multiple matches)
            let clicked = self.get_tower_near(local_x, local_y);
            // if we didnt click any tower, and we previously had a selected tower, deselect it
            if clicked.is_none() {
                if self.selected.is_some() {
                    self.selected = None;
                }
                return;
            }
            self.just_selected_tower = true;
            // if we did click a tower, and we previously didnt have a selected tower, select it
            let clicked = clicked.unwrap().0;
            if self.selected.is_none() {
                self.selected = Some(clicked);
                self.ui_manager.tower_open = true;
            } else {
                // if we did click a tower, and we previously did have a selected tower, check if theyre the same
                let old_selected = self.selected.unwrap();
                // if theyre not the same, move selection to the newly pressed tower
                if old_selected != clicked {
                    self.selected = Some(clicked);
                } else {
                    // if they are the same, start moving that tower
                    self.selected = None;
                    self.moving = Some(self.towers.remove(clicked));
                }
            }
        }
    }
    fn handle_ui(&mut self, local_x: f32, local_y: f32) {
        let selected_tower = self.selected.map(|index| &self.towers[index]);
        if let Some(tower) = selected_tower {
            self.icon_sheet.draw_tile(tower.x, tower.y, 32, false, 0.0);
            // draw little arrow thingy
            let x1 = (tower.x + SPRITE_SIZE / 2.0).round();
            let y1 = (tower.y + SPRITE_SIZE / 2.0).round();
            let x2 = (x1 + tower.direction.x * 16.0).round();
            let y2 = (y1 + tower.direction.y * 16.0).round();
            draw_line(x1, y1, x2, y2, 1.0, COLOR_YELLOW);
            for m in [-1.0, 1.0] {
                let x1 = x2;
                let y1 = y2;
                let angle = 40.0_f32.to_radians();
                let direction = Vec2::from_angle(PI + tower.direction.to_angle() + angle * m);
                let x2 = x1 + direction.x * 8.0;
                let y2 = y1 + direction.y * 8.0;
                draw_line(x1, y1, x2, y2, 1.0, COLOR_YELLOW);
            }

            let distance = ((x2 - local_x).powi(2) + (y2 - local_y).powi(2)).sqrt();
            let border_color = if distance <= 2.0 {
                COLOR_BEIGE
            } else {
                COLOR_BROWN
            };

            draw_circle(x2, y2, 2.0, border_color);
            draw_circle(x2, y2, 1.0, COLOR_YELLOW);
        }
        if let Some(tower) = &self.moving {
            self.icon_sheet
                .draw_tile(tower.x, tower.y - 4.0, tower.sprite, false, 0.0);
            self.icon_sheet.draw_tile(tower.x, tower.y, 33, false, 0.0);
            if !self.is_valid_tower_placement(tower.x, tower.y) {
                self.icon_sheet
                    .draw_tile(tower.x, tower.y - 4.0, 34, false, 0.0);
            }
        }
        let selected_tower = self.selected.map(|index| &mut self.towers[index]);
        self.ui_manager.handle_ui(
            local_x,
            local_y,
            &self.card_sheet,
            selected_tower,
            self.just_selected_tower,
        );
        self.just_selected_tower = false;

        // display topbar
        let mut cursor_x = 2.0;
        draw_square(0.0, 0.0, 64.0, 8.0);

        // show lives
        self.icon_sheet.draw_tile(cursor_x, 0.0, 40, false, 0.0);
        cursor_x += 6.0;
        self.ui_manager
            .text_engine
            .draw_text(cursor_x, 2.0, &self.lives.to_string(), 0);
        cursor_x += 4.0 * 4.0;

        // show gold counter
        self.icon_sheet.draw_tile(cursor_x, 0.0, 39, false, 0.0);
        cursor_x += 6.0;
        let gold_text = self.ui_manager.gold.to_string();
        self.ui_manager
            .text_engine
            .draw_text(cursor_x, 2.0, &gold_text, 0);
        cursor_x += 4.0 * (gold_text.len() as f32 + 1.0);

        // show round counter

        // change icon for the round counter if a round is in progress
        let round_icon = if self.round_manager.in_progress {
            38
        } else {
            37
        };
        self.icon_sheet
            .draw_tile(cursor_x, 0.0, round_icon, false, 0.0);
        cursor_x += 6.0;
        self.ui_manager.text_engine.draw_text(
            cursor_x,
            2.0,
            &self.round_manager.round.to_string(),
            0,
        );

        // draw start round button
        let text = "start round";
        let width = text.len() as f32 * 4.0 + 4.0;
        let x = (SCREEN_WIDTH - width) / 2.0;
        if !self.round_manager.in_progress
            && draw_button(
                &self.ui_manager.text_engine,
                x,
                0.0,
                width,
                8.0,
                local_x,
                local_y,
                text,
            )
        {
            self.start_round();
        }
    }
    fn draw_tower(&self, tower: &Tower) {
        let mut sprite = tower.sprite;
        let mut flipped = false;
        let angle = tower.direction.to_angle().to_degrees();
        let up_angle_span = 25.0;
        let down_angle_spawn = 45.0;
        if angle > -90.0 - up_angle_span && angle < -90.0 + up_angle_span {
            sprite += 2;
        } else if angle < 90.0 + down_angle_spawn && angle > 90.0 - down_angle_spawn {
            sprite += 1;
        } else if angle < 90.0 && angle > -90.0 {
            flipped = true;
        }
        self.icon_sheet
            .draw_tile(tower.x, tower.y, sprite, flipped, 0.0);
    }
    fn draw(&self) {
        self.tileset.draw_tilemap(&self.map.background);
        self.tileset.draw_tilemap(&self.map.out_of_bounds);
        self.tileset.draw_tilemap(&self.map.obstructions);
        for tower in self.towers.iter() {
            self.draw_tower(tower);
        }
        for enemy in &self.enemies {
            let extra_size = enemy.ty.size - 1;
            let ground_offset = 2.0 + extra_size as f32 * SPRITE_SIZE;
            let anim_frame = (enemy.state.score * enemy.ty.speed * enemy.ty.anim_speed) as usize
                % enemy.ty.anim_length;
            let mut flipped = false;
            if enemy.moving_left && enemy.ty.should_flip {
                flipped = true;
            }
            let (centre_x, _) = enemy.get_centre();
            for i in 0..enemy.ty.size {
                for j in 0..enemy.ty.size {
                    let mut sprite = enemy.ty.sprite + anim_frame * enemy.ty.size;
                    if flipped {
                        sprite += enemy.ty.size - j - 1;
                    } else {
                        sprite += j;
                    }
                    sprite += i * 32;
                    self.icon_sheet.draw_tile(
                        enemy.x + j as f32 * SPRITE_SIZE - extra_size as f32 * SPRITE_SIZE / 2.0,
                        enemy.y + i as f32 * SPRITE_SIZE - ground_offset,
                        sprite,
                        flipped,
                        0.0,
                    );
                }
            }
            if enemy.state.freeze_frames > 0 {
                for j in 0..enemy.ty.size {
                    self.particle_sheet.draw_tile(
                        enemy.x + j as f32 * SPRITE_SIZE - extra_size as f32 * SPRITE_SIZE / 2.0,
                        enemy.y + extra_size as f32 * SPRITE_SIZE - ground_offset,
                        32 + 9,
                        false,
                        0.0,
                    );
                }
            }
            if enemy.stun_frames > 0 {
                let anim_frame = enemy.stun_frames % 3;
                self.particle_sheet.draw_tile(
                    centre_x - SPRITE_SIZE / 2.0,
                    enemy.y - SPRITE_SIZE / 2.0,
                    32 + 13 + anim_frame as usize,
                    false,
                    0.0,
                );
            }
        }
        for projectile in self.projectiles.iter() {
            match &projectile.draw_type {
                ProjectileDrawType::Sprite(index, rotation_mode) => {
                    let rotation = match rotation_mode {
                        SpriteRotationMode::Direction => projectile.direction.to_angle(),
                        SpriteRotationMode::Spin => (15.0 - projectile.life % 30.0) / 15.0 * PI,
                        SpriteRotationMode::None => 0.0,
                    };
                    self.particle_sheet.draw_tile(
                        projectile.x - SPRITE_SIZE / 2.0,
                        projectile.y - SPRITE_SIZE / 2.0,
                        *index,
                        false,
                        rotation,
                    );
                }
                ProjectileDrawType::Particle(particle) => {
                    (particle.function)(
                        particle,
                        &ParticleContext {
                            x: projectile.x,
                            y: projectile.y,
                            origin_x: projectile.spawn_x,
                            origin_y: projectile.spawn_y,
                            direction: projectile.direction,
                        },
                        &self.particle_sheet,
                    );
                }
                _ => {}
            }
        }
        for (particle, ctx) in self.orphaned_particles.iter() {
            (particle.function)(particle, ctx, &self.particle_sheet);
        }
    }
    fn update_particles(&mut self) {
        self.orphaned_particles.retain_mut(|(projectile, _)| {
            projectile.life += 1;
            projectile.life < projectile.lifetime
        });
    }
    fn update_projectiles(&mut self) {
        let death_queue = self.projectiles.extract_if(.., |projectile| {
            if projectile.modifier_data.confetti_trail {
                self.orphaned_particles.push((
                    particle::CONFETTIS[rand::gen_range(0, particle::CONFETTIS.len())].clone(),
                    ParticleContext {
                        x: projectile.x,
                        y: projectile.y,
                        origin_x: projectile.spawn_x,
                        origin_y: projectile.spawn_y,
                        direction: projectile.direction,
                    },
                ));
            }
            projectile.x += projectile.direction.x * projectile.modifier_data.speed;
            projectile.y += projectile.direction.y * projectile.modifier_data.speed;
            projectile.life += 1.0;
            if projectile.ghost_frames > 0 {
                projectile.ghost_frames -= 1;
            }

            projectile.modifier_data.speed =
                projectile.modifier_data.speed.lerp(0.0, projectile.drag);

            if let ProjectileDrawType::Particle(particle) = &mut projectile.draw_type {
                particle.life += 1;
            }

            if projectile.modifier_data.homing && !projectile.straight {
                let dir = if projectile.modifier_data.smart_aim {
                    self.enemies.last().map(|enemy| {
                        Vec2::new(enemy.x - projectile.x, enemy.y - projectile.y).normalize()
                    })
                } else {
                    get_direction_nearest_enemy(&self.enemies, projectile.x, projectile.y)
                };
                if let Some(dir) = dir {
                    projectile.direction = dir;
                }
            }
            if projectile.modifier_data.boomerang {
                // make proj boomerang back towards towers
                let spawn_angle = Vec2::new(
                    projectile.spawn_x - projectile.x,
                    projectile.spawn_y - projectile.y,
                );
                let new = projectile.direction.lerp(spawn_angle, 0.005);
                projectile.direction = new
            }
            if projectile.modifier_data.snake {
                // make projectile follow cos wave
                let period = 0.6;
                let amp = 0.8;
                // calculate previous frames slither amount, such that we can get the projectile's base direction, without the slithering,
                // so we can apply the new slither amount to that
                let old = if projectile.life > 1.0 {
                    (period * (projectile.life - 1.0)).cos() * amp
                } else {
                    0.0
                };
                let amt = (period * projectile.life).cos() * amp;
                let angle = projectile.direction.to_angle();
                projectile.direction = Vec2::from_angle(angle - old + amt);
            }
            if !projectile.modifier_data.anti_piercing {
                // check if projectile hit any enemy
                for enemy in self.enemies.iter_mut() {
                    // check that enemy hasnt already been killed this frame
                    if enemy.health <= 0.0 {
                        continue;
                    }
                    let (enemy_x, enemy_y) = enemy.get_centre();

                    let distance = ((enemy_x - projectile.x).powi(2)
                        + (enemy_y - projectile.y).powi(2))
                    .sqrt();
                    if distance < 8.0 + projectile.extra_size {
                        // hit!
                        let mut damage = projectile.modifier_data.damage.clone();
                        // if projectile deals random damage, apply that
                        if let Some((min, max)) = projectile.random_damage {
                            let amount = rand::gen_range(min, max) as f32;

                            if let Some(amt) = damage.get_mut(&DamageType::Magic) {
                                *amt += amount;
                            } else {
                                damage.insert(DamageType::Magic, amount);
                            }
                        }
                        for (damage_type, mut amount) in damage {
                            match &enemy.ty.damage_resistance {
                                // skip damage of enemy if fully resistant
                                DamageResistance::Full(ty) => {
                                    if *ty == damage_type {
                                        continue;
                                    }
                                }
                                // halve damage if enemy partially resistant
                                DamageResistance::Partial(ty) => {
                                    if *ty == damage_type {
                                        amount /= 2.0;
                                    }
                                }
                                DamageResistance::None => {}
                            }
                            enemy.health -= amount;
                        }
                        if !projectile.modifier_data.damage.is_empty() {
                            enemy.gold_factor = projectile.modifier_data.gold_factor;
                        }
                        // play sound
                        projectile.hit_sound.play(&self.sfx_manager);
                        projectile.hit_sound = ProjectileSound::None;

                        // also check whether enemy should be frozen
                        // if projectile deals cold damage
                        if projectile
                            .modifier_data
                            .damage
                            .get(&DamageType::Cold)
                            .is_some_and(|f| *f > 0.0)
                        {
                            let is_cold_resistant = match enemy.ty.damage_resistance {
                                DamageResistance::None => false,
                                DamageResistance::Full(ty) => matches!(ty, DamageType::Cold),
                                DamageResistance::Partial(ty) => matches!(ty, DamageType::Cold),
                            };
                            // if enemy isnt cold resistant
                            if !is_cold_resistant {
                                enemy.state.freeze_frames = FREEZE_TIME;
                            }
                        }
                        // poison enemy if projectile has poison frames
                        if projectile.modifier_data.poison > 0 {
                            enemy.poison_frames = enemy
                                .poison_frames
                                .saturating_add(projectile.modifier_data.poison)
                                .min(30);
                        }
                        // stun enemy if projectile has stun frames
                        if projectile.modifier_data.stuns > 0 && enemy.stun_immunity_frames == 0 {
                            enemy.stun_frames = enemy
                                .stun_frames
                                .saturating_add(projectile.modifier_data.stuns)
                                .min(35);
                            enemy.stun_immunity_frames = STUN_IMMUNITY_FRAMES;
                        }

                        // send trigger payload
                        if !projectile.payload.is_empty() {
                            self.projectile_spawnlist
                                .append(&mut projectile.fire_payload());
                        }
                        // spawn hitmarker particle
                        self.orphaned_particles.push((
                            particle::HIT_MARKER,
                            ParticleContext {
                                x: projectile.x,
                                y: projectile.y,
                                origin_x: projectile.spawn_x,
                                origin_y: projectile.spawn_y,
                                direction: projectile.direction,
                            },
                        ));
                        if !projectile.modifier_data.piercing {
                            // kil projectile if not piercing
                            return true;
                        }
                    }
                }
            }

            // lifetime of -1.0 means projectile just doesnt despawn
            if projectile.modifier_data.lifetime != -1.0 {
                // if projectile is too old, kill it
                if projectile.life >= projectile.modifier_data.lifetime {
                    return true;
                }
            }
            // check for collisions
            if !projectile.modifier_data.ghost
                && projectile.ghost_frames == 0
                && projectile.x > 0.0
                && projectile.y > 0.0
            {
                let (x, y) = (
                    projectile.x as usize / SPRITE_SIZE_USIZE,
                    projectile.y as usize / SPRITE_SIZE_USIZE,
                );
                if y < self.map.obstructions.len()
                    && x < self.map.obstructions[0].len()
                    && self.map.obstructions[y][x] != 0
                {
                    // send trigger payload
                    if !projectile.payload.is_empty() && !projectile.only_enemy_triggers {
                        let mut spawnlist = projectile.fire_payload();
                        for p in spawnlist.iter_mut() {
                            let max_spread = (p.modifier_data.spread + DEFAULT_SPREAD).max(0.0);
                            let spread = rand::gen_range(-max_spread, max_spread);
                            // flip their direction, so payload is shot off like a bounce off the obstacle
                            let inverted =
                                Vec2::from_angle(PI + projectile.direction.to_angle() + spread);
                            p.direction = inverted;
                            p.ghost_frames = 10;
                        }
                        self.projectile_spawnlist.append(&mut spawnlist);
                    }
                    return true;
                }
            }
            false
        });
        for killed in death_queue.collect::<Vec<Projectile>>() {
            if !killed.death_payload.is_empty() {
                let mut context = FiringContext::default();
                fire_deck(
                    killed.x,
                    killed.y,
                    killed.direction,
                    killed.death_payload,
                    &mut context,
                );
                self.projectile_spawnlist.append(&mut context.spawn_list);
            }
            // if projectile had a particle thats still alive, orphan it
            if let ProjectileDrawType::Particle(particle) = killed.draw_type {
                if particle.life < particle.lifetime {
                    self.orphaned_particles.push((
                        particle,
                        ParticleContext {
                            x: killed.x,
                            y: killed.y,
                            origin_x: killed.spawn_x,
                            origin_y: killed.spawn_y,
                            direction: killed.direction,
                        },
                    ));
                }
            }
        }

        // spawn spawnlist

        // first iterate through spawnlist to update direction of projectiles with aiming
        // and play sfx
        for projectile in &mut self.projectile_spawnlist {
            if projectile.modifier_data.aim {
                let dir = if projectile.modifier_data.smart_aim {
                    self.enemies.last().map(|enemy| {
                        Vec2::new(enemy.x - projectile.x, enemy.y - projectile.y).normalize()
                    })
                } else {
                    get_direction_nearest_enemy(&self.enemies, projectile.x, projectile.y)
                };
                if let Some(direction_nearest) = dir {
                    let max_spread = projectile.modifier_data.spread.max(0.0);
                    let spread = rand::gen_range(-max_spread, max_spread);
                    projectile.direction = Vec2::from_angle(direction_nearest.to_angle() + spread);
                }
            }

            projectile.fire_sound.play(&self.sfx_manager);
        }
        self.projectiles.append(&mut self.projectile_spawnlist);
    }
    fn update_towers(&mut self, deltatime_ms: u128) {
        for tower in self.towers.iter_mut() {
            if !tower.can_shoot() {
                tower.delay_counter -= deltatime_ms as f32 / 1000.0;
            } else {
                let mut spawn_queue = tower.shoot();
                self.projectile_spawnlist.append(&mut spawn_queue);
            }
        }
    }
    fn update_state(&mut self) {
        if self.lives == 0 {
            // lose
            self.state = GameState::Lose;
        }
        if self.round_manager.round >= self.round_manager.rounds.len() {
            // win
            self.state = GameState::Win;
        }
    }
    fn update_enemies(&mut self) {
        if !self.round_manager.in_progress {
            return;
        }

        let mut spawnlist = Vec::new();

        let round_update = self.round_manager.update();
        if let RoundUpdate::Spawn(enemy) = &round_update {
            self.spawn_enemy(enemy);
        }

        self.enemies.retain_mut(|enemy| {
            if enemy.poison_frames > 0 {
                let mut damage = POISON_DAMAGE;
                match &enemy.ty.damage_resistance {
                    DamageResistance::None => {}
                    DamageResistance::Full(ty) => {
                        if *ty == DamageType::Acid {
                            damage = 0.0;
                        }
                    }
                    DamageResistance::Partial(ty) => {
                        if *ty == DamageType::Acid {
                            damage /= 2.0;
                        }
                    }
                }
                enemy.health -= damage;
            }
            if enemy.health <= 0.0 {
                self.ui_manager.gold +=
                    (enemy.ty.damage as f32 * 4.0 * enemy.gold_factor.unwrap_or(1.0)) as u16;
                if let EnemyPayload::Some(enemy_type, amount) = enemy.ty.payload {
                    for index in 0..amount {
                        let score = enemy.state.score + index as f32 * 2.0 - amount as f32 + 1.0;
                        let mut state = enemy.state;
                        state.score = score;
                        let Some((x, y)) = self.map.get_pos_along_path(score) else {
                            continue;
                        };
                        let new = Enemy::new(enemy_type, x * SPRITE_SIZE, y * SPRITE_SIZE, state);
                        spawnlist.push(new);
                    }
                }
                return false;
            }
            if let Some((x, y)) = self.map.get_pos_along_path(enemy.state.score) {
                if x * SPRITE_SIZE > enemy.x {
                    enemy.moving_left = false;
                } else if x * SPRITE_SIZE < enemy.x {
                    enemy.moving_left = true;
                }
                enemy.x = x * SPRITE_SIZE;
                enemy.y = y * SPRITE_SIZE;
            } else {
                self.lives = self.lives.saturating_sub(enemy.ty.calc_damage());
                return false;
            }
            let mut speed_factor = 1.0;
            if enemy.state.freeze_frames > 0 {
                enemy.state.freeze_frames -= 1;
                speed_factor = 0.55;
            }

            if enemy.stun_frames > 0 {
                enemy.stun_frames -= 1;
                // make larger enemies not stun entirely
                if enemy.ty.size == 1 {
                    speed_factor = 0.0
                } else {
                    speed_factor = 0.4
                }
            }
            // only subtract stun immunity frames after stun frames have wore off
            else if enemy.stun_immunity_frames > 0 {
                enemy.stun_immunity_frames -= 1;
            }
            enemy.state.score += enemy.ty.speed * speed_factor;
            true
        });

        self.enemies.append(&mut spawnlist);
        self.enemies
            .sort_by(|a, b| a.state.score.total_cmp(&b.state.score));

        if matches!(round_update, RoundUpdate::Finished) && self.enemies.is_empty() {
            self.ui_manager.gold += GOLD_ROUND_REWARD;
            self.round_manager.finish_round();
            if !self.lab {
                self.ui_manager.open_shop(
                    self.round_manager.round - 1,
                    DEFAULT_SHOP_SLOTS_HORIZONTAL,
                    DEFAULT_SHOP_SLOTS_VERTICAL,
                );
                // reward with new towers on special rounds
                for (round, index) in [(17, 2), (34, 3)] {
                    if self.round_manager.round == round {
                        let new = get_towers(self.map.tower_spawnpoints)[index].clone();
                        self.orphaned_particles.push((
                            particle::NEW_TOWER,
                            ParticleContext {
                                x: new.x,
                                y: new.y,
                                origin_x: new.x,
                                origin_y: new.y,
                                direction: LEFT,
                            },
                        ));
                        self.towers.push(new);
                        break;
                    }
                }

                // save
                let data = SaveData::create(self);
                write_save(data);
            } else {
                self.ui_manager.open_lab_shop();
            }
            // despawn all immortal projectiles so they dont carry over to next round,
            // because that would be kind of OP, allowing you to ex. build a larger and larger
            // heap of road thorns
            self.kill_immortal_projectiles();
        }
    }
    fn kill_immortal_projectiles(&mut self) {
        self.projectiles
            .retain(|f| f.modifier_data.lifetime != -1.0);
    }
}

struct GameManager {
    sludge: Option<Sludge>,
    in_play_menu: bool,
    maps: Vec<Map>,
    last: Instant,
    pixel_camera: Camera2D,
    gameover_anim_frame: u8,
    menu_texture: Texture2D,
    text_engine: TextEngine,
    tileset: Spritesheet,
}
impl GameManager {
    fn new() -> Self {
        let render_target = render_target(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
        render_target.texture.set_filter(FilterMode::Nearest);
        let menu_texture = assets::load_texture("data/assets/menu.png");
        Self {
            in_play_menu: false,
            sludge: None,
            maps: load_maps(),
            last: Instant::now(),
            pixel_camera: Camera2D {
                render_target: Some(render_target),
                zoom: Vec2::new(1.0 / SCREEN_WIDTH * 2.0, 1.0 / SCREEN_HEIGHT * 2.0),
                target: Vec2::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0),

                ..Default::default()
            },
            gameover_anim_frame: 0,
            menu_texture,
            text_engine: TextEngine::new(),
            tileset: load_spritesheet("data/assets/tileset.png", SPRITE_SIZE_USIZE),
        }
    }
    async fn run(&mut self) {
        loop {
            let (screen_width, screen_height) = screen_size();
            let scale_factor = (screen_width / SCREEN_WIDTH).min(screen_height / SCREEN_HEIGHT);

            let (mouse_x, mouse_y) = mouse_position();
            let local_x = mouse_x / scale_factor;
            let local_y = mouse_y / scale_factor;

            clear_background(BLACK);
            set_camera(&self.pixel_camera);

            if self.sludge.is_some() {
                self.run_game(local_x, local_y);
            } else if self.in_play_menu {
                self.run_play_menu(local_x, local_y).await
            } else {
                self.run_main_menu(local_x, local_y).await;
            }

            // draw low res render to screen
            set_default_camera();
            let render_target = &self.pixel_camera.render_target;
            let texture = render_target.clone().unwrap();
            draw_texture_ex(
                &texture.texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(
                        SCREEN_WIDTH * scale_factor,
                        SCREEN_HEIGHT * scale_factor,
                    )),
                    ..Default::default()
                },
            );
            next_frame().await;
        }
    }
    async fn run_play_menu(&mut self, local_x: f32, local_y: f32) {
        clear_background(WHITE);
        let text = "select map";
        self.text_engine.draw_text(
            SCREEN_WIDTH / 2.0 - text.len() as f32 * 4.0 / 2.0,
            2.0,
            text,
            0,
        );
        if draw_button(
            &self.text_engine,
            2.0,
            2.0,
            4.0 * 4.0 + 4.0,
            8.0,
            local_x,
            local_y,
            "back",
        ) || is_key_pressed(KeyCode::Escape)
        {
            self.in_play_menu = false;
        }
        let top_margin = 16.0;
        let horizontal_padding = 8.0;
        let vertical_padding = 16.0;
        let amt_horizontal = (SCREEN_WIDTH / (PREVIEW_WIDTH + horizontal_padding)) as usize;
        let left_padding =
            (SCREEN_WIDTH - (PREVIEW_WIDTH + horizontal_padding) * amt_horizontal as f32) / 2.0;
        for (index, map) in self.maps.iter().skip(1).enumerate() {
            let x = (index % amt_horizontal) as f32 * (PREVIEW_WIDTH + horizontal_padding)
                + left_padding;
            let y =
                (index / amt_horizontal) as f32 * (PREVIEW_HEIGHT + vertical_padding) + top_margin;
            if draw_button(
                &self.text_engine,
                x,
                y,
                PREVIEW_WIDTH + 4.0,
                PREVIEW_HEIGHT + 4.0 + 8.0,
                local_x,
                local_y,
                "",
            ) {
                let index = index + 1;
                // start game
                // create new Sludge instance
                let mut new = Sludge::new(
                    self.maps[index].clone(),
                    index,
                    self.text_engine.clone(),
                    false,
                    get_seed(),
                )
                .await;
                new.ui_manager.open_spawn_shop();
                self.sludge = Some(new);
                self.in_play_menu = false;
            }
            map.draw_preview(x + 2.0, y + 10.0, &self.pixel_camera, &self.tileset);
            self.text_engine.draw_text(x + 2.0, y + 2.0, &map.name, 2);
        }
    }
    async fn run_main_menu(&mut self, local_x: f32, local_y: f32) {
        draw_texture(&self.menu_texture, 0.0, 0.0, WHITE);

        let left_padding = 12.0;
        let top_padding = 26.0;

        let button_width = 60.0;
        let button_height = 8.0;

        draw_button_disabled(
            &self.text_engine,
            left_padding,
            top_padding,
            button_width,
            button_height,
            "load save",
        );
        if save_exists() {
            if let Some(save) = read_save() {
                if draw_button(
                    &self.text_engine,
                    left_padding,
                    top_padding,
                    button_width,
                    button_height,
                    local_x,
                    local_y,
                    "load save",
                ) {
                    let sludge = save.load(&self.maps, self.text_engine.clone()).await;
                    self.sludge = Some(sludge);
                }
            }
        }
        if draw_button(
            &self.text_engine,
            left_padding - 2.0,
            top_padding + button_height + 2.0,
            button_width,
            button_height,
            local_x,
            local_y,
            "play",
        ) {
            self.in_play_menu = true;
            return;
        }
        if draw_button(
            &self.text_engine,
            left_padding - 4.0,
            top_padding + (button_height + 2.0) * 2.0,
            button_width,
            button_height,
            local_x,
            local_y,
            "open lab",
        ) {
            let mut new = Sludge::new(
                self.maps[0].clone(),
                0,
                self.text_engine.clone(),
                true,
                get_seed(),
            )
            .await;
            new.ui_manager.open_lab_shop();
            self.sludge = Some(new);
        }
        if draw_button(
            &self.text_engine,
            left_padding - 6.0,
            top_padding + (button_height + 2.0) * 3.0,
            button_width,
            button_height,
            local_x,
            local_y,
            "exit",
        ) {
            std::process::exit(0);
        }
    }
    fn run_game(&mut self, local_x: f32, local_y: f32) {
        let Some(game) = &mut self.sludge else {
            panic!()
        };

        // run update loops if game is not over
        if let GameState::Running = game.state {
            game.handle_input(local_x, local_y);
            let now = Instant::now();
            let deltatime_ms = (now - self.last).as_millis();
            // run update loops at fixed FPS
            if deltatime_ms >= 1000 / 30 {
                self.last = now;
                game.update_projectiles();
                game.update_enemies();
                game.update_particles();
                game.update_towers(1000 / 30);
                game.update_state();
            }
        }

        // always draw
        game.draw();

        if is_key_pressed(KeyCode::Escape) {
            match game.state {
                GameState::Running => {
                    // pause game

                    // first try to place whatever tower we're moving so it isnt lost
                    // and if that spot is obstructed, default to the first tower spawnpoint
                    if let Some(mut moving) = game.moving.take() {
                        if !game.is_valid_tower_placement(moving.x, moving.y) {
                            let (x, y) = game.map.tower_spawnpoints[0];
                            let (x, y) = (x as f32, y as f32);
                            moving.x = x;
                            moving.y = y;
                        }
                        game.towers.push(moving);
                    }
                    // also check if we have a card on the cursor, if so, put it in the first available inventory slot
                    if let Some(card) = game.ui_manager.cursor_card.take() {
                        'outer: for row in &mut game.ui_manager.inventory {
                            for slot in row {
                                if slot.is_none() {
                                    *slot = Some(card);
                                    break 'outer;
                                }
                            }
                        }
                    }
                    game.state = GameState::Paused
                }
                GameState::Paused => game.state = GameState::Running,
                _ => {}
            }
        }

        match game.state {
            GameState::Running => {
                // only draw ui if game is not over
                game.handle_ui(local_x, local_y);
            }
            _ => {
                let mut anim_frame = self.gameover_anim_frame;
                let header = match game.state {
                    GameState::Win => "    you win",
                    GameState::Lose => "   you lose",
                    GameState::Paused => {
                        anim_frame = 30;
                        "paused"
                    }
                    _ => {
                        panic!()
                    }
                };
                let width = 64.0;
                let height = 64.0;
                let y =
                    anim_frame as f32 / 30.0 * (SCREEN_HEIGHT / 2.0 + height / 2.0) - height - 1.0;
                let x = SCREEN_WIDTH / 2.0 - width / 2.0;
                ui::draw_square(x, y, width, height);
                if !matches!(game.state, GameState::Paused) && self.gameover_anim_frame < 30 {
                    self.gameover_anim_frame += 1;
                }
                game.ui_manager
                    .text_engine
                    .draw_text(x + 2.0, y + 4.0, header, 1);
                let text = format!(
                    "lives: {}\ngold: {}\nround: {}",
                    game.lives, game.ui_manager.gold, game.round_manager.round
                );
                game.ui_manager
                    .text_engine
                    .draw_text(x + 2.0, y + 4.0 + 8.0, &text, 2);

                let button_width = 60.0;
                let button_height = 8.0;
                let button_x = x + width / 2.0 - button_width / 2.0;
                let button_y = y + height - 4.0 - button_height;

                let clicked = draw_button(
                    &game.ui_manager.text_engine,
                    button_x,
                    button_y,
                    button_width,
                    button_height,
                    local_x,
                    local_y,
                    "return to menu",
                );

                if clicked {
                    // save the game if we're paused and exiting to menu
                    if let GameState::Paused = game.state {
                        if !game.lab
                            && game.round_manager.round > 0
                            && !game.round_manager.in_progress
                            && game.ui_manager.shop.is_some()
                        {
                            let data = SaveData::create(game);
                            write_save(data);
                        }
                    }
                    // delete save if player just lost/won the game and exiting to menu
                    else {
                        remove_save()
                    }
                    self.sludge = None;
                }
            }
        }
    }
}

#[macroquad::main("sludge")]
async fn main() {
    let mut game_manager = GameManager::new();
    game_manager.run().await;
}
