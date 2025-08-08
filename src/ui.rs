use macroquad::prelude::*;

use crate::{
    cards::{get_cards, Card},
    consts::*,
    map::Spritesheet,
    tower::Tower,
};
enum InventorySlot {
    Inventory(usize, usize),
    Tower(usize),
}
pub struct UIManager {
    inventory: [[Option<Card>; INV_SLOTS_HORIZONTAL]; INV_SLOTS_VERTICAL],
    pub inventory_open: bool,
    cursor_card: Option<Card>,
}
impl UIManager {
    pub fn new(grant_all_cards: bool) -> Self {
        let mut inventory = std::array::from_fn(|_| std::array::from_fn(|_| None.clone()).clone());
        if grant_all_cards {
            let all_cards = get_cards();
            for (index, card) in all_cards.into_iter().enumerate() {
                inventory[index / inventory[0].len()][index % inventory[0].len()] = Some(card);
            }
        }
        Self {
            inventory,
            inventory_open: false,
            cursor_card: None,
        }
    }
    pub fn get_menu_handle_state(&self) -> (f32, f32, bool) {
        if self.inventory_open {
            (
                (SCREEN_WIDTH - MENU_WIDTH - SPRITE_SIZE),
                (SCREEN_HEIGHT / 2.0 - SPRITE_SIZE),
                true,
            )
        } else {
            (
                (SCREEN_WIDTH - SPRITE_SIZE),
                (SCREEN_HEIGHT / 2.0 - SPRITE_SIZE),
                false,
            )
        }
    }

    pub fn draw_ui(
        &self,
        local_x: f32,
        local_y: f32,
        card_sheet: &Spritesheet,
        icon_sheet: &Spritesheet,
        selected_tower: Option<&Tower>,
    ) {
        if let Some(tower) = selected_tower {
            for (index, card_slot) in tower.card_slots.iter().enumerate() {
                // todo: draw text
                let tile_x = index as f32 * CARD_SIZE;
                let tile_y = 8.0;
                if let Some(card) = card_slot {
                    card.draw(card_sheet, tile_x as f32 + 2.0, tile_y + 2.0);
                } else {
                    draw_square(tile_x, tile_y, CARD_SIZE, CARD_SIZE);
                }
            }
        }
        if self.inventory_open {
            draw_square(SCREEN_WIDTH - MENU_WIDTH, 0.0, MENU_WIDTH, SCREEN_HEIGHT);
            for y in 0..self.inventory.len() {
                for x in 0..self.inventory[0].len() {
                    let tile_x = SCREEN_WIDTH - MENU_WIDTH + 2.0 + x as f32 * CARD_SIZE;
                    let tile_y = 2.0 + y as f32 * CARD_SIZE;
                    if let Some(card) = &self.inventory[y][x] {
                        card.draw(card_sheet, tile_x + 2.0, tile_y + 2.0);
                    } else {
                        draw_square(tile_x, tile_y, CARD_SIZE, CARD_SIZE);
                    }
                }
            }
        }
        let (handle_x, handle_y, flipped) = self.get_menu_handle_state();
        icon_sheet.draw_tile(handle_x, handle_y, 35, flipped, 0.0);

        if let Some(card) = &self.cursor_card {
            let x = local_x - SPRITE_SIZE / 2.0;
            let y = local_y - SPRITE_SIZE / 2.0;

            card.draw(card_sheet, x, y);
        }
    }
    fn get_hovered_slot(
        &self,
        local_x: f32,
        local_y: f32,
        slots_amt: usize,
    ) -> Option<InventorySlot> {
        if self.inventory_open
            && local_x > SCREEN_WIDTH - MENU_WIDTH + 2.0
            && local_x < SCREEN_WIDTH - 3.0
            && local_y > 2.0
        {
            let tile_x =
                (local_x as usize + MENU_WIDTH_USIZE - SCREEN_WIDTH_USIZE - 2) / CARD_SIZE_USIZE;
            let tile_y = (local_y as usize - 2) / CARD_SIZE_USIZE;

            if tile_y < INV_SLOTS_VERTICAL {
                return Some(InventorySlot::Inventory(tile_x, tile_y));
            }
        } else if local_y > 8.0 && local_y < 8.0 + SPRITE_SIZE + 4.0 && slots_amt > 0 {
            let tile_x = local_x as usize / CARD_SIZE_USIZE;
            if tile_x < slots_amt {
                return Some(InventorySlot::Tower(tile_x));
            }
        }
        None
    }
    pub fn handle_input(
        &mut self,
        local_x: f32,
        local_y: f32,
        selected_tower: Option<&mut Tower>,
    ) -> bool {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return false;
        }
        let (handle_x, handle_y, _) = self.get_menu_handle_state();
        if local_x > handle_x
            && local_x < handle_x + SPRITE_SIZE
            && local_y > handle_y
            && local_y < handle_y + SPRITE_SIZE
        {
            self.inventory_open = !self.inventory_open;
            return true;
        }

        let tower_card_slots = match selected_tower {
            None => None,
            Some(tower) => Some(&mut tower.card_slots),
        };

        let slots_amt = tower_card_slots.as_ref().map(|f| f.len()).unwrap_or(0);

        if let Some(slot) = self.get_hovered_slot(local_x, local_y, slots_amt) {
            match slot {
                InventorySlot::Inventory(x, y) => {
                    std::mem::swap(&mut self.inventory[y][x], &mut self.cursor_card);
                }
                InventorySlot::Tower(x) => {
                    std::mem::swap(&mut tower_card_slots.unwrap()[x], &mut self.cursor_card);
                }
            }
            return true;
        }
        false
    }
}

pub fn draw_square(x: f32, y: f32, w: f32, h: f32) {
    let x = x as f32;
    let y = y as f32;
    let w = w as f32;
    let h = h as f32;
    draw_rectangle(x, y, w, h, UI_OUTER_BORDER_COLOR);
    draw_rectangle(x + 1.0, y + 1.0, w - 2.0, h - 2.0, UI_BG_COLOR);
}
pub fn draw_text(x: f32, y: f32, text: &str) {
    let x = x as f32;
    let y = y as f32 + 5.0;
    macroquad::text::draw_text(text, x, y, 8.0, BLACK);
}
