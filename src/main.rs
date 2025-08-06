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
    *source_x == target_x && *source_y == target_y
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
    inventory: [[Option<Card>; (MENU_WIDTH - 4) / 11]; (SCREEN_HEIGHT - 4) / 11],
    card_inventory_open: bool,
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
        let mut tower1 = TOWERS[0].clone();
        tower1.x = map.tower_spawnpoints[0].0;
        tower1.y = map.tower_spawnpoints[0].1;

        let mut tower2 = TOWERS[1].clone();
        tower2.x = map.tower_spawnpoints[1].0;
        tower2.y = map.tower_spawnpoints[1].1;

        Self {
            map,
            enemies: Vec::new(),
            towers: vec![tower1, tower2],
            lives: STARTING_LIVES,
            round: 0,
            round_in_progress: false,
            moving: None,
            selected: None,
            inventory: [[None; (MENU_WIDTH - 4) / 11]; (SCREEN_HEIGHT - 4) / 11],
            card_inventory_open: false,
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
    /// Returns whether a UI element was interacted with
    fn handle_ui_input(&mut self, scale_factor: usize, local_x: f32, local_y: f32) -> bool {
        let local_x = local_x as usize;
        let local_y = local_y as usize;

        let (handle_x, handle_y, _) = self.get_menu_handle_state();
        if local_x > handle_x
            && local_x < handle_x + SPRITE_SIZE
            && local_y > handle_y
            && local_y < handle_y + SPRITE_SIZE
            && is_mouse_button_pressed(MouseButton::Left)
        {
            self.card_inventory_open = !self.card_inventory_open;
            return true;
        }
        false
    }
    fn get_menu_handle_state(&self) -> (usize, usize, bool) {
        if self.card_inventory_open {
            (
                SCREEN_WIDTH - MENU_WIDTH - SPRITE_SIZE,
                SCREEN_HEIGHT / 2 - SPRITE_SIZE,
                true,
            )
        } else {
            (
                SCREEN_WIDTH - SPRITE_SIZE,
                SCREEN_HEIGHT / 2 - SPRITE_SIZE,
                false,
            )
        }
    }
    fn handle_input(&mut self, scale_factor: usize) {
        let (mouse_x, mouse_y) = mouse_position();
        let local_x = mouse_x / scale_factor as f32;
        let local_y = mouse_y / scale_factor as f32;

        // if we're currently dragging a tower
        if self.moving.is_some() {
            let mut tower_x = 0;
            let mut tower_y = 0;
            if let Some(tower) = &mut self.moving {
                // set tower x and y to cursor position (offset by 4 so its centered, and clamped to be inside map)
                tower.x = (local_x as usize)
                    .saturating_sub(SPRITE_SIZE / 2)
                    .min(SCREEN_WIDTH - 1 - SPRITE_SIZE);
                tower.y = (local_y as usize).min(SCREEN_HEIGHT - 1 - SPRITE_SIZE);

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
        if self.handle_ui_input(scale_factor, local_x, local_y) {
            return;
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            // find tower being clicked by iterating through all towers and checking if distance to cursor <= 8.0
            // (also sorted based on distance for multiple matches)
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
            // if we didnt click any tower, and we previously had a selected tower, deselect it
            if clicked.is_none() {
                if self.selected.is_some() {
                    self.selected = None;
                    self.card_inventory_open = false;
                }
                return;
            }
            // if we did click a tower, and we previously didnt have a selected tower, select it
            let clicked = clicked.unwrap().0;
            if self.selected.is_none() {
                self.selected = Some(clicked);
                self.card_inventory_open = true;
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
    fn draw_ui(&self, scale_factor: usize) {
        if let Some(selected) = self.selected {
            let tower = &self.towers[selected];
            for (index, card_slot) in tower.card_slots.iter().enumerate() {
                // todo: draw text
                let tile_x = index * (SPRITE_SIZE + 3);
                let tile_y = 8;
                ui::draw_square(
                    scale_factor,
                    tile_x,
                    tile_y,
                    SPRITE_SIZE + 4,
                    SPRITE_SIZE + 4,
                );
                if let Some(card) = card_slot {
                    self.cards
                        .draw_tile(scale_factor, tile_x, tile_y, card.sprite, false);
                }
            }
        }
        if self.card_inventory_open {
            ui::draw_body(
                scale_factor,
                SCREEN_WIDTH - MENU_WIDTH,
                0,
                MENU_WIDTH,
                SCREEN_HEIGHT,
            );
            for y in 0..self.inventory.len() {
                for x in 0..self.inventory[0].len() {
                    ui::draw_square(
                        scale_factor,
                        SCREEN_WIDTH - MENU_WIDTH + 2 + x * 11,
                        2 + y * 11,
                        12,
                        12,
                    );
                }
            }
        }
        let (handle_x, handle_y, flipped) = self.get_menu_handle_state();
        self.icons
            .draw_tile(scale_factor, handle_x, handle_y, 35, flipped);

        if let Some(selected) = self.selected {
            let tower = &self.towers[selected];
            self.icons
                .draw_tile(scale_factor, tower.x, tower.y, 32, false);
        }
        if let Some(tower) = &self.moving {
            self.icons.draw_tile(
                scale_factor,
                tower.x,
                tower.y.saturating_sub(4),
                tower.sprite,
                false,
            );
            self.icons
                .draw_tile(scale_factor, tower.x, tower.y, 33, false);
            if !self.is_valid_tower_placement(tower.x, tower.y) {
                self.icons
                    .draw_tile(scale_factor, tower.x, tower.y.saturating_sub(4), 34, false);
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
                false,
            );
        }
        for tower in self.towers.iter() {
            self.icons
                .draw_tile(scale_factor, tower.x, tower.y, tower.sprite, false);
        }
        self.draw_ui(scale_factor)
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

        for (remove_offset, index) in death_queue.iter().enumerate() {
            self.enemies.remove(index - remove_offset);
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

        game.handle_input(scale_factor);

        // run update loops at fixed 30 FPS
        if time_since_last >= 1000 / 30 {
            last = now;
            game.update_enemies();
        }

        // always draw
        game.draw(scale_factor);

        // debug
        if is_key_pressed(KeyCode::Space) {
            game.round_in_progress = !game.round_in_progress;
        }

        next_frame().await;
    }
}
