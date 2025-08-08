use macroquad::prelude::*;

use crate::{
    assets::load_spritesheet,
    cards::{get_cards, Card, CardType},
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
    font: Spritesheet,
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
            font: load_spritesheet("data/assets/font.png", 4),
        }
    }
    pub fn draw_text(&self, local_x: f32, mut local_y: f32, text: &str, color_offset: usize) {
        let mut i = 0;
        for char in text.chars() {
            if char == ' ' {
                i += 1;
                continue;
            }
            if char == '\n' {
                local_y += 5.0;
                i = 0;
                continue;
            }
            let mut index = char as usize;

            // hardcoded index of '.'
            if char == '.' {
                index = 'z' as usize - 'a' as usize + 12;
            }

            // for characters in range a-z
            if index >= 'a' as usize {
                index -= 'a' as usize
            }
            // for characters 0-9
            else if index >= '0' as usize {
                index -= '0' as usize;
                index += 'z' as usize - 'a' as usize + 1;
            }
            index += color_offset * self.font.width / 4;
            self.font
                .draw_tile(local_x + 4.0 * i as f32, local_y, index, false, 0.0);
            i += 1;
        }
    }
    fn draw_card_info(
        &self,
        mut local_x: f32,
        local_y: f32,
        card: &Card,
        card_sheet: &Spritesheet,
    ) {
        let width = 64.0 + 32.0;
        if local_x > SCREEN_WIDTH / 2.0 {
            local_x -= width;
        }
        draw_square(local_x, local_y, width, 32.0);
        card.draw(card_sheet, local_x + 4.0, local_y + 4.0);
        let mut name = card.name.to_string();
        if card.is_trigger {
            name += " trigger"
        }
        self.draw_text(local_x + 4.0 + CARD_SIZE, local_y + 3.0, &name, 1);
        self.draw_text(local_x + 4.0 + CARD_SIZE, local_y + 8.0, card.desc, 0);
        let modifier_data = match &card.ty {
            CardType::Modifier(modifier_data) => modifier_data,
            CardType::Projectile(proj, _) => &proj.modifier_data,
            _ => {
                return;
            }
        };

        let mut index = 0.0;
        for (k, v) in modifier_data.iter() {
            self.draw_text(
                local_x + 2.0,
                local_y + index * 5.0 + CARD_SIZE + 4.0,
                &format!("{k}:{v}"),
                2,
            );
            index += 1.0;
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
            let width = (tower.card_slots.len() as f32 * CARD_SIZE + 4.0).max(68.0);
            draw_square(0.0, 8.0, width, CARD_SIZE + 4.0 + 5.0 * 3.0);
            let tile_y = 8.0 + 2.0;
            for (index, (k, v)) in [
                ("shoot delay", tower.shoot_delay),
                ("reload time", tower.recharge_speed),
            ]
            .iter()
            .enumerate()
            {
                self.draw_text(
                    2.0,
                    tile_y + CARD_SIZE + 5.0 + 5.0 * index as f32,
                    &format!("{k}:{v}"),
                    2,
                );
            }
            for (index, card_slot) in tower.card_slots.iter().enumerate() {
                // todo: draw text
                let tile_x = index as f32 * CARD_SIZE + 2.0;
                if let Some(card) = card_slot {
                    card.draw(card_sheet, tile_x + 2.0, tile_y + 2.0);
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
        let handle_sprite = if local_x > handle_x
            && local_x < handle_x + SPRITE_SIZE
            && local_y > handle_y
            && local_y < handle_y + SPRITE_SIZE
        {
            36
        } else {
            35
        };
        icon_sheet.draw_tile(handle_x, handle_y, handle_sprite, flipped, 0.0);

        if let Some(card) = &self.cursor_card {
            let x = local_x - SPRITE_SIZE / 2.0;
            let y = local_y - SPRITE_SIZE / 2.0;

            card.draw(card_sheet, x, y);
        } else {
            let slots_amt = selected_tower.map(|f| f.card_slots.len()).unwrap_or(0);
            let hovered = self.get_hovered_slot(local_x, local_y, slots_amt);
            if let Some(hovered) = hovered {
                let card = match hovered {
                    InventorySlot::Inventory(x, y) => &self.inventory[y][x],
                    InventorySlot::Tower(x) => &selected_tower.unwrap().card_slots[x],
                };
                if let Some(card) = card {
                    self.draw_card_info(local_x, local_y, card, card_sheet);
                }
            }
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
        } else if local_x > 2.0
            && local_y > 8.0
            && local_y < 8.0 + SPRITE_SIZE + 4.0
            && slots_amt > 0
        {
            let tile_x = (local_x as usize - 2) / CARD_SIZE_USIZE;
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
    draw_rectangle(x, y, w, h, UI_BORDER_COLOR);
    draw_rectangle(x + 1.0, y + 1.0, w - 2.0, h - 2.0, UI_BG_COLOR);
}
pub fn draw_button(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, UI_BORDER_COLOR);
    draw_rectangle(x + 1.0, y + 1.0, w - 2.0, h - 2.0, UI_BUTTON_BG_COLOR);
}
