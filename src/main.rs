use std::f32::consts::PI;
use std::time::Instant;

use crate::cards::*;
use crate::consts::*;
use crate::enemy::*;
use crate::map::*;
use crate::particle::Particle;
use crate::rounds::*;
use crate::tower::*;
use crate::ui::*;
use macroquad::{miniquad::window::screen_size, prelude::*};

mod cards;
mod consts;
mod enemy;
mod map;
mod particle;
mod rounds;
mod tower;
mod ui;

/// Move source x and y towards target x and y with speed. Returns if the target was reached/hit.
fn move_towards(
    speed: f32,
    source_x: &mut f32,
    source_y: &mut f32,
    target_x: f32,
    target_y: f32,
) -> bool {
    if *source_x < target_x {
        *source_x += speed;
    } else if *source_x > target_x {
        *source_x -= speed;
    }
    if *source_y < target_y {
        *source_y += speed;
    } else if *source_y > target_y {
        *source_y -= speed;
    }
    *source_x == target_x && *source_y == target_y
}

async fn load_spritesheet(path: &str) -> Spritesheet {
    let error = format!("{} is missing!!", path);
    Spritesheet::new(load_texture(path).await.expect(&error))
}

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

struct Sludge {
    map: Map,
    enemies: Vec<Enemy>,
    towers: Vec<Tower>,
    projectiles: Vec<Projectile>,
    orphaned_particles: Vec<(Particle, f32, f32)>,
    lives: u8,
    ui_manager: UIManager,
    round_manager: RoundManager,
    moving: Option<Tower>,
    selected: Option<usize>,
    tileset: Spritesheet,
    icon_sheet: Spritesheet,
    card_sheet: Spritesheet,
    particle_sheet: Spritesheet,
}
impl Sludge {
    async fn new(map: Map) -> Self {
        let tileset = load_spritesheet("spritesheet.png").await;
        let icon_sheet = load_spritesheet("icons.png").await;
        let card_sheet = load_spritesheet("cards.png").await;
        let particle_sheet = load_spritesheet("particles.png").await;

        // add starting towers
        let base_towers = get_towers();
        let mut tower1 = base_towers[0].clone();
        tower1.x = map.tower_spawnpoints[0].0 as f32;
        tower1.y = map.tower_spawnpoints[0].1 as f32;
        tower1.direction = LEFT;

        let mut tower2 = base_towers[1].clone();
        tower2.x = map.tower_spawnpoints[1].0 as f32;
        tower2.y = map.tower_spawnpoints[1].1 as f32;
        tower2.direction = LEFT;

        Self {
            map,
            enemies: Vec::with_capacity(100),
            towers: vec![tower1, tower2],
            projectiles: Vec::with_capacity(100),
            orphaned_particles: Vec::with_capacity(100),
            lives: STARTING_LIVES,
            round_manager: load_round_data(),
            moving: None,
            selected: None,
            ui_manager: UIManager::new(true),
            tileset,
            icon_sheet,
            card_sheet,
            particle_sheet,
        }
    }
    fn spawn_enemy(&mut self, ty: &'static EnemyType) {
        let spawn = self.map.points[0];
        let enemy = Enemy {
            ty,
            x: spawn.0 * SPRITE_SIZE,
            y: spawn.1 * SPRITE_SIZE,
            health: ty.max_health,
            next_path_point: 1,
            score: 0.0,
            moving_left: false,
        };
        self.enemies.push(enemy);
    }
    fn is_valid_tower_placement(&self, x: f32, y: f32) -> bool {
        for tower in &self.towers {
            let distance =
                ((tower.x as f32 - x as f32).powi(2) + (tower.y as f32 - y as f32).powi(2)).sqrt();
            if distance < SPRITE_SIZE as f32 {
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
            let distance = ((tower.x as f32 + SPRITE_SIZE as f32 / 2.0 - local_x as f32).powi(2)
                + (tower.y as f32 + SPRITE_SIZE as f32 / 2.0 - local_y as f32).powi(2))
            .sqrt();
            if distance <= SPRITE_SIZE as f32 {
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
                tower.x = (local_x - SPRITE_SIZE / 2.0)
                    .min(SCREEN_WIDTH - 1.0 - SPRITE_SIZE)
                    .max(0.0);
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
                    let mut spawn_queue = tower.shoot(LEFT);
                    self.projectiles.append(&mut spawn_queue);
                }
            }
        }
    }

    fn draw(&self, local_x: f32, local_y: f32) {
        self.tileset.draw_tilemap(&self.map.background);
        self.tileset.draw_tilemap(&self.map.obstructions);
        for tower in self.towers.iter() {
            self.icon_sheet
                .draw_tile(tower.x, tower.y, tower.sprite, false, 0.0);
        }
        for enemy in &self.enemies {
            let anim_frame = (enemy.score * enemy.ty.speed) as usize % enemy.ty.anim_length;
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
        }
        for projectile in self.projectiles.iter() {
            match &projectile.draw_type {
                ProjectileDrawType::Sprite(index, rotation_mode) => {
                    let rotation = match rotation_mode {
                        SpriteRotationMode::Direction => projectile.direction.to_angle(),
                        SpriteRotationMode::Spin => {
                            (15.0 - projectile.life % 30.0) as f32 / 15.0 * PI
                        }
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
                        projectile.direction,
                        &self.particle_sheet,
                    );
                }
                _ => {}
            }
        }
        for (particle, x, y) in self.orphaned_particles.iter() {
            (particle.function)(particle, *x, *y, LEFT, &self.particle_sheet);
        }
        let selected_tower = match self.selected {
            Some(index) => Some(&self.towers[index]),
            None => None,
        };
        self.ui_manager.draw_ui(
            local_x,
            local_y,
            &self.card_sheet,
            &self.icon_sheet,
            selected_tower,
        );
        if let Some(tower) = selected_tower {
            self.icon_sheet.draw_tile(tower.x, tower.y, 32, false, 0.0);
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
    }
    fn update_particles(&mut self) {
        let mut death_queue = Vec::new();
        for (index, (particle, _, _)) in self.orphaned_particles.iter_mut().enumerate() {
            particle.life += 1;
            if particle.life >= particle.lifetime {
                // kill the orhpan
                death_queue.push(index);
            }
        }
        for (remove_offset, index) in death_queue.iter().enumerate() {
            self.orphaned_particles.remove(index - remove_offset);
        }
    }
    fn update_projectiles(&mut self) {
        let mut death_queue = Vec::new();
        let mut new_projectiles = Vec::new();

        for (index, projectile) in self.projectiles.iter_mut().enumerate() {
            projectile.x += (projectile.direction.x * projectile.modifier_data.speed as f32) as f32;
            projectile.y += (projectile.direction.y * projectile.modifier_data.speed as f32) as f32;
            projectile.life += 1.0;

            projectile.modifier_data.speed =
                projectile.modifier_data.speed.lerp(0.0, projectile.drag);

            if let ProjectileDrawType::Particle(particle) = &mut projectile.draw_type {
                particle.life += 1;
            }
            let mut dead = false;
            if projectile.life >= projectile.modifier_data.lifetime {
                dead = true;
            }
            if projectile.modifier_data.homing {
                let target_dir =
                    get_direction_nearest_enemy(&self.enemies, projectile.x, projectile.y);
                if let Some(target_dir) = target_dir {
                    projectile.direction = target_dir;
                }
            }
            // check if projectile hit any enemy
            for enemy in self.enemies.iter_mut() {
                let distance = ((enemy.x as f32 - projectile.x as f32).powi(2)
                    + (enemy.y as f32 - projectile.y as f32).powi(2))
                .sqrt();
                if distance < 8.0 + projectile.extra_size {
                    // hit!
                    for (damage_type, mut amount) in projectile.modifier_data.damage.clone() {
                        match &enemy.ty.damage_resistance {
                            DamageResistance::Full(ty) => {
                                if *ty == damage_type {
                                    continue;
                                }
                            }
                            DamageResistance::Partial(ty) => {
                                if *ty == damage_type {
                                    amount /= 2.0;
                                }
                            }
                            DamageResistance::None => {}
                        }
                        enemy.health -= amount;
                    }
                    let direction_nearest_enemy =
                        Vec2::new(enemy.x - projectile.x, enemy.y - projectile.y).normalize();
                    // send trigger payload
                    if !projectile.payload.is_empty() {
                        let mut context = FiringContext::default();
                        fire_deck(
                            projectile.x,
                            projectile.y,
                            projectile.direction,
                            direction_nearest_enemy,
                            projectile.payload.clone(),
                            &mut context,
                        );
                        new_projectiles.append(&mut context.spawn_list);
                    }
                    // spawn hitmarker particle
                    self.orphaned_particles.push((
                        particle::HIT_MARKER,
                        projectile.x,
                        projectile.y,
                    ));
                    if !projectile.modifier_data.piercing {
                        // kil projectile if not piercing
                        dead = true;
                        break;
                    }
                }
            }
            if dead {
                death_queue.push(index);
            }
        }
        for (remove_offset, index) in death_queue.iter().enumerate() {
            let killed = self.projectiles.remove(index - remove_offset);
            if !killed.death_payload.is_empty() {
                let direction_nearest_enemy =
                    get_direction_nearest_enemy(&self.enemies, killed.x, killed.y);
                let mut context = FiringContext::default();
                fire_deck(
                    killed.x,
                    killed.y,
                    killed.direction,
                    direction_nearest_enemy.unwrap_or(LEFT),
                    killed.death_payload,
                    &mut context,
                );
                self.projectiles.append(&mut context.spawn_list);
            }
            if let ProjectileDrawType::Particle(particle) = killed.draw_type {
                if particle.life < particle.lifetime {
                    self.orphaned_particles.push((particle, killed.x, killed.y));
                }
            }
        }
        self.projectiles.append(&mut new_projectiles);
    }
    fn update_towers(&mut self, deltatime_ms: u128) {
        for tower in self.towers.iter_mut() {
            if !tower.can_shoot() {
                tower.delay_counter -= deltatime_ms as f32 / 1000.0;
            } else {
                let direction_nearest_enemy =
                    get_direction_nearest_enemy(&self.enemies, tower.x, tower.y);
                if self.round_manager.in_progress && !self.enemies.is_empty() {
                    let mut spawn_queue =
                        tower.shoot(direction_nearest_enemy.unwrap_or(tower.direction));
                    self.projectiles.append(&mut spawn_queue);
                }
            }
        }
    }
    fn update_enemies(&mut self) {
        if !self.round_manager.in_progress {
            return;
        }
        let round_update = self.round_manager.update();
        if let RoundUpdate::Spawn(enemy) = &round_update {
            self.spawn_enemy(enemy);
        }

        let mut death_queue = Vec::new();

        for (index, enemy) in self.enemies.iter_mut().enumerate() {
            if enemy.health <= 0.0 {
                death_queue.push(index);
                continue;
            }
            let next_x = self.map.points[enemy.next_path_point].0 * SPRITE_SIZE;
            let next_y = self.map.points[enemy.next_path_point].1 * SPRITE_SIZE;
            if next_x > enemy.x {
                enemy.moving_left = false;
            } else if next_x < enemy.x {
                enemy.moving_left = true;
            }

            // move enemy towards next path point. if point is reached, increment next path point index
            if move_towards(enemy.ty.speed, &mut enemy.x, &mut enemy.y, next_x, next_y) {
                enemy.next_path_point += 1;
                // if at last path point, kill this enemy
                if enemy.next_path_point >= self.map.points.len() {
                    death_queue.push(index);
                }
            }
            enemy.score += enemy.ty.speed;
        }

        for (remove_offset, index) in death_queue.iter().enumerate() {
            self.enemies.remove(index - remove_offset);
        }
        if matches!(round_update, RoundUpdate::Finished) && self.enemies.len() == 0 {
            self.round_manager.finish_round();
        }
    }
}

#[macroquad::main("sludge")]
async fn main() {
    let mut scale_factor;
    let maps = load_maps();
    let mut game = Sludge::new(maps[0].clone()).await;

    let mut last = Instant::now();

    let render_target = render_target(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
    render_target.texture.set_filter(FilterMode::Nearest);

    let low_res_camera = Camera2D {
        render_target: Some(render_target),
        zoom: Vec2::new(
            1.0 / SCREEN_WIDTH as f32 * 2.0,
            1.0 / SCREEN_HEIGHT as f32 * 2.0,
        ),
        target: Vec2::new(SCREEN_WIDTH as f32 / 2.0, SCREEN_HEIGHT as f32 / 2.0),

        ..Default::default()
    };
    loop {
        // update scale factor
        let (screen_width, screen_height) = screen_size();
        scale_factor =
            (screen_width as f32 / SCREEN_WIDTH).min(screen_height as f32 / SCREEN_HEIGHT);
        clear_background(BLACK);
        set_camera(&low_res_camera);

        let now = Instant::now();
        let deltatime_ms = (now - last).as_millis();

        let (mouse_x, mouse_y) = mouse_position();
        let local_x = mouse_x as f32 / scale_factor;
        let local_y = mouse_y as f32 / scale_factor;

        game.handle_input(local_x, local_y);

        // run update loops at fixed FPS
        if deltatime_ms >= 1000 / 30 {
            last = now;
            game.update_enemies();
            game.update_projectiles();
            game.update_particles();
            game.update_towers(deltatime_ms);
        }

        // always draw
        game.draw(local_x, local_y);

        // draw low res render to screen
        set_default_camera();
        let render_target = &low_res_camera.render_target;
        let texture = render_target.clone().unwrap();
        draw_texture_ex(
            &texture.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(
                    (SCREEN_WIDTH * scale_factor) as f32,
                    (SCREEN_HEIGHT * scale_factor) as f32,
                )),
                ..Default::default()
            },
        );

        // debug
        if is_key_pressed(KeyCode::E) {
            game.round_manager.in_progress = !game.round_manager.in_progress;
        }

        next_frame().await;
    }
}
