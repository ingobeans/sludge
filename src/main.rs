use std::time::Instant;

use crate::cards::*;
use crate::consts::*;
use crate::map::*;
use macroquad::{miniquad::window::screen_size, prelude::*};

mod cards;
mod consts;
mod map;
mod ui;

enum DamageType {
    Magic,
    Pierce,
    Explosion,
    Cold,
    Acid,
}
/// Struct that holds information about an enemy type
struct EnemyType {
    sprite: usize,
    anim_length: usize,
    speed: usize,
    damage_resistance: Vec<DamageType>,
}
/// A live instance of an enemy
struct Enemy {
    ty: &'static EnemyType,
    x: usize,
    y: usize,
    next_path_point: usize,
    score: usize,
}

const ENEMY_TYPES: &[EnemyType] = &[
    // spider
    EnemyType {
        sprite: 2 * 32,
        anim_length: 2,
        speed: 1,
        damage_resistance: Vec::new(),
    },
];

/// Move source x and y towards target x and y with speed. Returns if the target was reached/hit.
fn move_towards(
    speed: usize,
    source_x: &mut usize,
    source_y: &mut usize,
    target_x: usize,
    target_y: usize,
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
    return *source_x == target_x && *source_y == target_y;
}

async fn load_spritesheet(path: &str) -> Spritesheet {
    let error = format!("{} is missing!!", path);
    Spritesheet::new(load_texture(path).await.expect(&error))
}

struct Sludge {
    map: Map,
    enemies: Vec<Enemy>,
    towers: Vec<Tower>,
    lives: u8,
    round: u8,
    round_in_progress: bool,
    moving: Option<Tower>,
    selected: Option<usize>,
    tileset: Spritesheet,
    icons: Spritesheet,
    cards: Spritesheet,
}
impl Sludge {
    async fn new(map: Map) -> Self {
        let tileset = load_spritesheet("spritesheet.png").await;
        let icons = load_spritesheet("icons.png").await;
        let cards = load_spritesheet("cards.png").await;

        // add starting towers
        let tower1 = Tower {
            x: map.tower_spawnpoints[0].0,
            y: map.tower_spawnpoints[0].1,
            sprite: 0,
            cards: Vec::new(),
            card_index: 0,
            shoot_delay: 0.32,
            delay_counter: 0.0,
            direction: Direction::LEFT,
        };
        let tower2 = Tower {
            x: map.tower_spawnpoints[1].0,
            y: map.tower_spawnpoints[1].1,
            sprite: 3,
            cards: Vec::new(),
            card_index: 0,
            shoot_delay: 0.32,
            delay_counter: 0.0,
            direction: Direction::LEFT,
        };

        Self {
            map,
            enemies: Vec::new(),
            towers: vec![tower1, tower2],
            lives: STARTING_LIVES,
            round: 0,
            round_in_progress: false,
            moving: None,
            selected: None,
            tileset,
            icons,
            cards,
        }
    }
    fn spawn_enemy(&mut self, ty: &'static EnemyType) {
        let spawn = self.map.points[0];
        let enemy = Enemy {
            ty,
            x: spawn.0 * SPRITE_SIZE,
            y: spawn.1 * SPRITE_SIZE,
            next_path_point: 1,
            score: 0,
        };
        self.enemies.push(enemy);
    }
    fn is_valid_tower_placement(&self, x: usize, y: usize) -> bool {
        for tower in &self.towers {
            let distance =
                ((tower.x as f32 - x as f32).powi(2) + (tower.y as f32 - y as f32).powi(2)).sqrt();
            if distance < SPRITE_SIZE as f32 {
                return false;
            }
        }
        self.map.is_unobstructed(x, y)
    }
    fn handle_input(&mut self, scale_factor: usize) {
        let (mouse_x, mouse_y) = mouse_position();
        let local_x = mouse_x / scale_factor as f32;
        let local_y = mouse_y / scale_factor as f32;
        if self.moving.is_some() {
            let mut tower_x = 0;
            let mut tower_y = 0;
            if let Some(tower) = &mut self.moving {
                tower.x = (local_x as usize)
                    .saturating_sub(SPRITE_SIZE / 2)
                    .min(SCREEN_WIDTH - 1 - SPRITE_SIZE);
                tower.y = (local_y as usize).min(SCREEN_HEIGHT - 1 - SPRITE_SIZE);

                tower_x = tower.x;
                tower_y = tower.y;
            }
            let valid = self.is_valid_tower_placement(tower_x, tower_y);
            if !is_mouse_button_down(MouseButton::Left) && valid {
                self.towers.push(self.moving.take().unwrap());
            }
            return;
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            // find if we're clicking tower
            let mut clicked = None;
            for (index, tower) in self.towers.iter().enumerate() {
                let distance = ((tower.x as f32 + SPRITE_SIZE as f32 / 2.0 - local_x).powi(2)
                    + (tower.y as f32 + SPRITE_SIZE as f32 / 2.0 - local_y).powi(2))
                .sqrt();
                if distance <= 8.0 {
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
            if clicked.is_none() {
                if self.selected.is_some() {
                    self.selected = None;
                }
                return;
            }
            let clicked = clicked.unwrap().0;
            if self.selected.is_none() {
                self.selected = Some(clicked);
            } else {
                let old_selected = self.selected.unwrap();
                if old_selected != clicked {
                    self.selected = Some(clicked);
                } else {
                    self.selected = None;
                    self.moving = Some(self.towers.remove(clicked));
                }
            }
        }
    }
    fn draw(&self, scale_factor: usize) {
        self.tileset
            .draw_tilemap(scale_factor, &self.map.background);
        self.tileset
            .draw_tilemap(scale_factor, &self.map.obstructions);
        for enemy in &self.enemies {
            let anim_frame = enemy.score / enemy.ty.speed % enemy.ty.anim_length;
            self.icons.draw_tile(
                scale_factor,
                enemy.x,
                enemy.y,
                enemy.ty.sprite + anim_frame,
                0.0,
            );
        }
        for (index, tower) in self.towers.iter().enumerate() {
            self.icons
                .draw_tile(scale_factor, tower.x, tower.y, tower.sprite, 0.0);
            if let Some(selected) = self.selected {
                if selected == index {
                    self.icons
                        .draw_tile(scale_factor, tower.x, tower.y, 32, 0.0);
                }
            }
        }
        if let Some(tower) = &self.moving {
            self.icons.draw_tile(
                scale_factor,
                tower.x,
                tower.y.saturating_sub(4),
                tower.sprite,
                0.0,
            );
            self.icons
                .draw_tile(scale_factor, tower.x, tower.y, 33, 0.0);
            if !self.is_valid_tower_placement(tower.x, tower.y) {
                self.icons
                    .draw_tile(scale_factor, tower.x, tower.y.saturating_sub(4), 34, 0.0);
            }
        }
    }
    fn update_enemies(&mut self) {
        if !self.round_in_progress {
            return;
        }
        let mut death_queue = Vec::new();

        for (index, enemy) in self.enemies.iter_mut().enumerate() {
            let next_x = self.map.points[enemy.next_path_point].0 * SPRITE_SIZE;
            let next_y = self.map.points[enemy.next_path_point].1 * SPRITE_SIZE;

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

        let mut remove_offset = 0;
        for index in death_queue {
            self.enemies.remove(index - remove_offset);
            remove_offset += 1;
        }
        if self.enemies.len() == 0 {
            self.round_in_progress = false;
            self.round += 1;
        }
    }
}

#[macroquad::main("sludge")]
async fn main() {
    let mut scale_factor;
    let maps = load_maps();
    let mut game = Sludge::new(maps[0].clone()).await;
    game.spawn_enemy(&ENEMY_TYPES[0]);

    let mut last = Instant::now();

    loop {
        // update scale factor
        let (screen_width, screen_height) = screen_size();
        scale_factor =
            (screen_width as usize / SCREEN_WIDTH).min(screen_height as usize / SCREEN_HEIGHT);
        clear_background(BLACK);

        let now = Instant::now();
        let time_since_last = (now - last).as_millis();

        // run update loops at fixed 30 FPS
        if time_since_last >= 1000 / 30 {
            last = now;
            game.update_enemies();
        }

        // always draw
        game.draw(scale_factor);

        game.handle_input(scale_factor);
        // debug
        if is_key_pressed(KeyCode::Space) {
            game.round_in_progress = !game.round_in_progress;
        }

        next_frame().await;
    }
}
