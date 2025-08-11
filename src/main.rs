#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]

use std::collections::VecDeque;
use std::f32::consts::PI;
use std::time::Instant;

use crate::assets::*;
use crate::cards::*;
use crate::consts::*;
use crate::enemy::*;
use crate::map::*;
use crate::particle::Particle;
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

fn get_direction_nearest_enemy(enemies: &Vec<Enemy>, x: f32, y: f32) -> Option<Vec2> {
    if enemies.is_empty() {
        return None;
    }
    let mut nearest: (f32, Vec2) = (f32::MAX, Vec2::ZERO);
    for enemy in enemies {
        let distance = ((enemy.x - x).powi(2) + (enemy.y - y).powi(2)).sqrt();
        if distance < nearest.0 {
            nearest = (distance, Vec2::new(enemy.x - x, enemy.y - y).normalize())
        }
    }
    Some(nearest.1)
}
enum GameState {
    Running,
    Win,
    Lose,
    Paused,
}
struct Sludge {
    state: GameState,
    map: Map,
    map_index: usize,
    enemies: Vec<Enemy>,
    enemy_spawn_queue: VecDeque<(&'static EnemyType, f32, f32, EnemyState)>,
    towers: Vec<Tower>,
    projectiles: Vec<Projectile>,
    projectile_spawnlist: Vec<Projectile>,
    orphaned_particles: Vec<(Particle, f32, f32, Vec2)>,
    lives: u8,
    ui_manager: UIManager,
    round_manager: RoundManager,
    sfx_manager: SFXManager,
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
    async fn new(map: Map, map_index: usize, text_engine: TextEngine, lab: bool) -> Self {
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
        let round_manager = load_round_data();
        let sfx_manager = SFXManager::new().await;

        Self {
            state: GameState::Running,
            map,
            map_index,
            enemies: Vec::with_capacity(100),
            enemy_spawn_queue: VecDeque::with_capacity(10),
            towers,
            projectiles: Vec::with_capacity(100),
            projectile_spawnlist: Vec::with_capacity(100),
            orphaned_particles: Vec::with_capacity(100),
            lives: STARTING_LIVES,
            round_manager,
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
        self.kill_immortal_projectiles();
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
    /// Returns whether a UI element was interacted with
    fn handle_ui_input(&mut self, local_x: f32, local_y: f32) -> bool {
        let selected_tower = match self.selected {
            None => None,
            Some(index) => Some(&mut self.towers[index]),
        };
        self.ui_manager
            .handle_input(local_x, local_y, selected_tower)
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
                if !is_mouse_button_down(MouseButton::Left) {
                    self.rotating_tower = false;
                }
                return;
            }

            if is_mouse_button_pressed(MouseButton::Left) {
                let distance = ((x2 - local_x).powi(2) + (y2 - local_y).powi(2)).sqrt();
                if distance <= 2.0 {
                    self.rotating_tower = true;
                    return;
                }
            }
        }

        // handle ui elements. if an element is interacted with, also return to stop other inputs being handled.
        if self.handle_ui_input(local_x, local_y) {
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
                    self.ui_manager.inventory_open = false;
                }
                return;
            }
            // if we did click a tower, and we previously didnt have a selected tower, select it
            let clicked = clicked.unwrap().0;
            if self.selected.is_none() {
                self.selected = Some(clicked);
                self.ui_manager.inventory_open = true;
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
        if is_key_down(KeyCode::Space) {
            if let Some(selected) = self.selected {
                let tower = &mut self.towers[selected];
                if tower.can_shoot() && !self.round_manager.in_progress {
                    let mut spawn_queue = tower.shoot();
                    self.projectile_spawnlist.append(&mut spawn_queue);
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
        self.ui_manager
            .draw_ui(local_x, local_y, &self.card_sheet, selected_tower);
        // display topbar
        let mut cursor_x = 0.0;

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
            let anim_frame = (enemy.state.score * enemy.ty.speed * enemy.ty.anim_speed) as usize
                % enemy.ty.anim_length;
            let mut flipped = false;
            if enemy.moving_left && enemy.ty.should_flip {
                flipped = true;
            }
            for i in 0..enemy.ty.size {
                for j in 0..enemy.ty.size {
                    let mut sprite = enemy.ty.sprite + anim_frame * enemy.ty.size;
                    if flipped {
                        sprite += enemy.ty.size - j - 1;
                    } else {
                        sprite += j;
                    }
                    sprite += i * 32;
                    let ground_offset = 2.0 + (enemy.ty.size - 1) as f32 * SPRITE_SIZE;
                    self.icon_sheet.draw_tile(
                        enemy.x + j as f32 * SPRITE_SIZE,
                        enemy.y + i as f32 * SPRITE_SIZE - ground_offset,
                        sprite,
                        flipped,
                        0.0,
                    );
                }
            }
            if enemy.state.freeze_frames > 0 {
                let extra_size = enemy.ty.size - 1;
                let ground_offset = 2.0 + extra_size as f32 * SPRITE_SIZE;
                self.particle_sheet.draw_tile(
                    enemy.x + extra_size as f32 * SPRITE_SIZE,
                    enemy.y + extra_size as f32 * SPRITE_SIZE - ground_offset,
                    32 + 9,
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
                        projectile.x,
                        projectile.y,
                        *index,
                        false,
                        rotation,
                    );
                }
                ProjectileDrawType::Particle(particle) => {
                    (particle.function)(
                        particle,
                        projectile.x,
                        projectile.y,
                        &projectile.direction,
                        &self.particle_sheet,
                    );
                }
                _ => {}
            }
        }
        for (particle, x, y, direction) in self.orphaned_particles.iter() {
            (particle.function)(particle, *x, *y, direction, &self.particle_sheet);
        }
    }
    fn update_particles(&mut self) {
        self.orphaned_particles.retain_mut(|(projectile, _, _, _)| {
            projectile.life += 1;
            projectile.life < projectile.lifetime
        });
    }
    fn update_projectiles(&mut self) {
        let death_queue = self.projectiles.extract_if(.., |projectile| {
            projectile.x += projectile.direction.x * projectile.modifier_data.speed;
            projectile.y += projectile.direction.y * projectile.modifier_data.speed;
            projectile.life += 1.0;

            projectile.modifier_data.speed =
                projectile.modifier_data.speed.lerp(0.0, projectile.drag);

            if let ProjectileDrawType::Particle(particle) = &mut projectile.draw_type {
                particle.life += 1;
            }

            if projectile.modifier_data.homing && !projectile.straight {
                let target_dir =
                    get_direction_nearest_enemy(&self.enemies, projectile.x, projectile.y);
                if let Some(target_dir) = target_dir {
                    projectile.direction = target_dir;
                }
            }
            // check if projectile hit any enemy
            for enemy in self.enemies.iter_mut() {
                // check that enemy hasnt already been killed this frame
                if enemy.health <= 0.0 {
                    continue;
                }

                let distance =
                    ((enemy.x - projectile.x).powi(2) + (enemy.y - projectile.y).powi(2)).sqrt();
                if distance < 8.0 + projectile.extra_size {
                    // hit!
                    for (damage_type, mut amount) in projectile.modifier_data.damage.clone() {
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
                    // if projectile deals random damage, apply that
                    if let Some((min, max)) = projectile.random_damage {
                        let amount = rand::gen_range(min, max) as f32;
                        enemy.health -= amount;
                    }
                    // play sound
                    projectile.hit_sound.play(&self.sfx_manager);
                    projectile.hit_sound = ProjectileSound::None;

                    // also check whether enemy should be frozen
                    // if projectile deals cold damage
                    if *projectile
                        .modifier_data
                        .damage
                        .get(&DamageType::Cold)
                        .unwrap_or(&0.0)
                        > 0.0
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
                    // stun enemy if projectile has stun frames
                    if projectile.stuns > 0 {
                        enemy.state.stun_frames =
                            enemy.state.stun_frames.saturating_add(projectile.stuns);
                        let mut particle = particle::STUNNED;
                        particle.lifetime = projectile.stuns;
                        self.orphaned_particles.push((
                            particle,
                            enemy.x,
                            enemy.y,
                            projectile.direction,
                        ));
                    }

                    // send trigger payload
                    if !projectile.payload.is_empty() {
                        self.projectile_spawnlist
                            .append(&mut projectile.fire_payload());
                    }
                    // spawn hitmarker particle
                    self.orphaned_particles.push((
                        particle::HIT_MARKER,
                        projectile.x,
                        projectile.y,
                        projectile.direction,
                    ));
                    if !projectile.modifier_data.piercing {
                        // kil projectile if not piercing
                        return true;
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
            if !projectile.ghost && projectile.x > 0.0 && projectile.y > 0.0 {
                let (x, y) = (
                    (projectile.x + SPRITE_SIZE / 2.0) as usize / SPRITE_SIZE_USIZE,
                    (projectile.y + SPRITE_SIZE / 2.0) as usize / SPRITE_SIZE_USIZE,
                );
                if y < self.map.obstructions.len()
                    && x < self.map.obstructions[0].len()
                    && self.map.obstructions[y][x] != 0
                {
                    // send trigger payload
                    if !projectile.payload.is_empty() {
                        self.projectile_spawnlist
                            .append(&mut projectile.fire_payload());
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
                    self.orphaned_particles
                        .push((particle, killed.x, killed.y, killed.direction));
                }
            }
        }

        // spawn spawnlist

        // first iterate through spawnlist to update direction of projectiles with aiming
        // and play sfx
        for projectile in &mut self.projectile_spawnlist {
            if projectile.modifier_data.aim {
                let direction_nearest =
                    get_direction_nearest_enemy(&self.enemies, projectile.x, projectile.y)
                        .unwrap_or(LEFT);
                projectile.direction = direction_nearest;
            }

            projectile.fire_sound.play(&self.sfx_manager);
        }
        self.projectiles.append(&mut self.projectile_spawnlist);
    }
    fn update_towers(&mut self, deltatime_ms: u128) {
        for tower in self.towers.iter_mut() {
            if !tower.can_shoot() {
                tower.delay_counter -= deltatime_ms as f32 / 1000.0;
            } else if self.round_manager.in_progress {
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

        if let Some((ty, x, y, state)) = self.enemy_spawn_queue.pop_front() {
            let enemy = Enemy::new(ty, x, y, state);
            self.enemies.push(enemy);
        }

        let round_update = self.round_manager.update();
        if let RoundUpdate::Spawn(enemy) = &round_update {
            self.spawn_enemy(enemy);
        }

        self.enemies.retain_mut(|enemy| {
            if enemy.health <= 0.0 {
                self.ui_manager.gold += enemy.ty.damage as u16 * 4;
                if let EnemyPayload::Some(enemy_type, amount) = enemy.ty.payload {
                    for _ in 0..amount {
                        self.enemy_spawn_queue.push_back((
                            enemy_type,
                            enemy.x,
                            enemy.y,
                            enemy.state.clone(),
                        ));
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
                speed_factor = 0.25;
            }
            if enemy.state.stun_frames > 0 {
                enemy.state.stun_frames -= 1;
                speed_factor = 0.0;
            }
            enemy.state.score += enemy.ty.speed * speed_factor;
            true
        });

        if matches!(round_update, RoundUpdate::Finished) && self.enemies.is_empty() {
            self.ui_manager.gold += GOLD_ROUND_REWARD;
            self.round_manager.finish_round();
            if !self.lab {
                self.ui_manager.open_shop(
                    self.round_manager.round - 1,
                    DEFAULT_SHOP_SLOTS_HORIZONTAL,
                    DEFAULT_SHOP_SLOTS_VERTICAL,
                );
            } else {
                self.ui_manager.open_lab_shop();
            }
            // despawn all immortal projectiles so they dont carry over to next round,
            // because that would be kind of OP, allowing you to ex. build a larger and larger
            // heap of road thorns
            self.kill_immortal_projectiles();

            // save
            let data = SaveData::create(self);
            write_save(data);
        }
    }
    fn kill_immortal_projectiles(&mut self) {
        self.projectiles
            .retain(|f| f.modifier_data.lifetime != -1.0);
    }
}

struct GameManager {
    sludge: Option<Sludge>,
    maps: Vec<Map>,
    last: Instant,
    pixel_camera: Camera2D,
    gameover_anim_frame: u8,
    menu_texture: Texture2D,
    text_engine: TextEngine,
}
impl GameManager {
    fn new() -> Self {
        let render_target = render_target(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
        render_target.texture.set_filter(FilterMode::Nearest);
        let menu_texture = assets::load_texture("data/assets/menu.png");
        Self {
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
            // start game
            // create new Sludge instance
            let mut new =
                Sludge::new(self.maps[1].clone(), 1, self.text_engine.clone(), false).await;
            new.ui_manager.open_spawn_shop();
            self.sludge = Some(new);
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
            let mut new =
                Sludge::new(self.maps[0].clone(), 0, self.text_engine.clone(), true).await;
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
                GameState::Running => game.state = GameState::Paused,
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
                    if let GameState::Paused = game.state {
                        if game.round_manager.round > 0
                            && !game.round_manager.in_progress
                            && game.ui_manager.shop.is_some()
                        {
                            let data = SaveData::create(game);
                            write_save(data);
                        }
                    }
                    self.sludge = None;
                }
            }
        }
    }
}

#[macroquad::main("sludge")]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as _);
    let mut game_manager = GameManager::new();
    game_manager.run().await;
}
