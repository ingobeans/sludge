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
use crate::tower::*;
use crate::ui::*;
use macroquad::{miniquad::window::screen_size, prelude::*};

mod assets;
mod cards;
mod consts;
mod enemy;
mod map;
mod particle;
mod rounds;
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
    enemies: Vec<Enemy>,
    enemy_spawn_queue: VecDeque<(&'static EnemyType, f32, f32, f32)>,
    towers: Vec<Tower>,
    projectiles: Vec<Projectile>,
    orphaned_particles: Vec<(Particle, f32, f32)>,
    lives: u8,
    gold: u16,
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
    fn new(map: Map, text_engine: TextEngine, lab: bool) -> Self {
        let tileset = load_spritesheet("data/assets/tileset.png", SPRITE_SIZE_USIZE);
        let icon_sheet = load_spritesheet("data/assets/icons.png", SPRITE_SIZE_USIZE);
        let card_sheet = load_spritesheet("data/assets/cards.png", SPRITE_SIZE_USIZE);
        let particle_sheet = load_spritesheet("data/assets/particles.png", SPRITE_SIZE_USIZE);

        // add starting towers
        let base_towers = get_towers(map.tower_spawnpoints);

        Self {
            state: GameState::Running,
            map,
            enemies: Vec::with_capacity(100),
            enemy_spawn_queue: VecDeque::with_capacity(10),
            towers: base_towers[..2].into(),
            projectiles: Vec::with_capacity(100),
            orphaned_particles: Vec::with_capacity(100),
            lives: STARTING_LIVES,
            gold: STARTING_GOLD,
            round_manager: load_round_data(),
            moving: None,
            selected: None,
            ui_manager: UIManager::new(lab, text_engine),
            tileset,
            icon_sheet,
            card_sheet,
            particle_sheet,
        }
    }
    fn spawn_enemy(&mut self, ty: &'static EnemyType) {
        let spawn = self.map.points[0];
        let enemy = Enemy::new(ty, spawn.0 * SPRITE_SIZE, spawn.1 * SPRITE_SIZE, 0.0);
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

    fn draw_ui(&self, local_x: f32, local_y: f32) {
        let selected_tower = self.selected.map(|index| &self.towers[index]);
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
        self.ui_manager.draw_ui(
            local_x,
            local_y,
            &self.card_sheet,
            &self.icon_sheet,
            selected_tower,
        );
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
        let gold_text = self.gold.to_string();
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
    }
    fn draw(&self) {
        self.tileset.draw_tilemap(&self.map.background);
        self.tileset.draw_tilemap(&self.map.obstructions);
        for tower in self.towers.iter() {
            self.icon_sheet
                .draw_tile(tower.x, tower.y, tower.sprite, false, 0.0);
        }
        for enemy in &self.enemies {
            let anim_frame = (enemy.score * enemy.ty.speed * enemy.ty.anim_speed) as usize
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
            projectile.x += projectile.direction.x * projectile.modifier_data.speed;
            projectile.y += projectile.direction.y * projectile.modifier_data.speed;
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
                let distance =
                    ((enemy.x - projectile.x).powi(2) + (enemy.y - projectile.y).powi(2)).sqrt();
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

        if let Some((ty, x, y, score)) = self.enemy_spawn_queue.pop_front() {
            let enemy = Enemy::new(ty, x, y, score);
            self.enemies.push(enemy);
        }

        let round_update = self.round_manager.update();
        if let RoundUpdate::Spawn(enemy) = &round_update {
            self.spawn_enemy(enemy);
        }

        let mut death_queue = Vec::new();

        for (index, enemy) in self.enemies.iter_mut().enumerate() {
            if enemy.health <= 0.0 {
                death_queue.push(index);
                self.gold += enemy.ty.damage as u16;
                if let EnemyPayload::Some(enemy_type, amount) = enemy.ty.payload {
                    for _ in 0..amount {
                        self.enemy_spawn_queue.push_back((
                            enemy_type,
                            enemy.x,
                            enemy.y,
                            enemy.score,
                        ));
                    }
                }
                continue;
            }
            if let Some((x, y)) = self.map.get_pos_along_path(enemy.score) {
                if x * SPRITE_SIZE > enemy.x {
                    enemy.moving_left = false;
                } else if x * SPRITE_SIZE < enemy.x {
                    enemy.moving_left = true;
                }
                enemy.x = x * SPRITE_SIZE;
                enemy.y = y * SPRITE_SIZE;
            } else {
                self.lives -= enemy.ty.calc_damage();
                death_queue.push(index);
            }
            enemy.score += enemy.ty.speed;
        }

        for (remove_offset, index) in death_queue.iter().enumerate() {
            self.enemies.remove(index - remove_offset);
        }
        if matches!(round_update, RoundUpdate::Finished) && self.enemies.is_empty() {
            self.round_manager.finish_round();
        }
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
                self.run_main_menu(local_x, local_y);
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
    fn run_main_menu(&mut self, local_x: f32, local_y: f32) {
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
            self.sludge = Some(Sludge::new(
                self.maps[1].clone(),
                self.text_engine.clone(),
                false,
            ))
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
            self.sludge = Some(Sludge::new(
                self.maps[0].clone(),
                self.text_engine.clone(),
                true,
            ))
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
                game.update_enemies();
                game.update_projectiles();
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

        // debug
        if is_key_pressed(KeyCode::E) {
            game.round_manager.in_progress = !game.round_manager.in_progress;
        }

        match game.state {
            GameState::Running => {
                // only draw ui if game is not over
                game.draw_ui(local_x, local_y);
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
                if !matches!(game.state, GameState::Paused) {
                    if self.gameover_anim_frame < 30 {
                        self.gameover_anim_frame += 1;
                    }
                }
                game.ui_manager
                    .text_engine
                    .draw_text(x + 2.0, y + 4.0, header, 1);
                let text = format!(
                    "lives: {}\ngold: {}\nround: {}",
                    game.lives, game.gold, game.round_manager.round
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
