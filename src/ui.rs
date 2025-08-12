use macroquad::prelude::*;

use crate::{
    assets::load_spritesheet,
    cards::{get_cards, get_random_shop_card, library, Card, CardType},
    consts::*,
    map::Spritesheet,
    tower::Tower,
};
enum InventorySlot {
    Inventory(usize, usize),
    Tower(usize),
}
#[derive(Clone)]
pub struct TextEngine {
    font: Spritesheet,
}
impl TextEngine {
    pub fn new() -> Self {
        Self {
            font: load_spritesheet("data/assets/font.png", 4),
        }
    }
    pub fn draw_text(&self, x: f32, mut y: f32, text: &str, color_offset: usize) {
        let mut i = 0;
        for char in text.chars() {
            if char == ' ' {
                i += 1;
                continue;
            }
            if char == '\n' {
                y += 5.0;
                i = 0;
                continue;
            }
            let mut index = char as usize;

            // hardcoded index of '.'
            if char == '.' {
                index = 'z' as usize - 'a' as usize + 12;
            }

            // hardcoded index of '-'
            if char == '-' {
                index = 'z' as usize - 'a' as usize + 13;
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
                .draw_tile(x + 4.0 * i as f32, y, index, false, 0.0);
            i += 1;
        }
    }
}

pub struct Shop {
    pub cards: Vec<Vec<Option<(Card, u16)>>>,
    pub open: bool,
}

pub struct UIManager {
    pub inventory: [[Option<Card>; INV_SLOTS_HORIZONTAL]; INV_SLOTS_VERTICAL],
    pub inventory_open: bool,
    cursor_card: Option<Card>,
    pub shop: Option<Shop>,
    pub gold: u16,
    pub text_engine: TextEngine,
}
impl UIManager {
    pub fn new(text_engine: TextEngine) -> Self {
        let inventory = std::array::from_fn(|_| std::array::from_fn(|_| None.clone()).clone());
        Self {
            inventory,
            inventory_open: false,
            cursor_card: None,
            shop: None,
            gold: STARTING_GOLD,
            text_engine,
        }
    }
    pub fn open_lab_shop(&mut self) {
        let was_open = self.shop.as_ref().is_some_and(|f| f.open);
        let mut shop_cards: Vec<Vec<Option<(Card, u16)>>> = Vec::new();
        let all_cards = get_cards();
        let slots_horizontal = 8;
        for (index, card) in all_cards.into_iter().enumerate() {
            let y = index / slots_horizontal;
            if y >= shop_cards.len() {
                shop_cards.push(vec![None; slots_horizontal]);
            }
            shop_cards[y][index % slots_horizontal] = Some((card, 0));
        }
        self.shop = Some(Shop {
            cards: shop_cards,
            open: was_open,
        });
    }
    pub fn open_spawn_shop(&mut self) {
        self.shop = Some(Shop {
            cards: vec![vec![None; 4]; 2],
            open: true,
        });
        let shop = self.shop.as_mut().unwrap();
        let mut cards = vec![
            (library::road_thorns(), 150),
            (library::icecicle(), 150),
            (library::thorn_dart(), 150),
            (library::rocket(), 150),
            (library::bomb(), 100),
            (library::dart(), 100),
            (library::magicbolt(), 100),
            (library::aiming(), 50),
        ];

        for row in shop.cards.iter_mut() {
            for slot in row.iter_mut() {
                if let Some((popped, price)) = cards.pop() {
                    *slot = Some((popped, price));
                }
            }
        }
    }
    pub fn open_shop(&mut self, round: usize, width: usize, height: usize) {
        let price_modifier = 1.0 + round as f32 / 7.5;
        let projectile_penalty = 1.2 + round as f32 / 40.0;
        let cards = get_cards();
        let mut shop_cards: Vec<Vec<Option<(Card, u16)>>> = Vec::new();
        for _ in 0..height {
            let mut row = Vec::with_capacity(width);
            for _ in 0..width {
                let card = get_random_shop_card(round, &cards);
                let mut price = rand::gen_range(120.0, 210.0);
                if let CardType::Projectile(_, _) = &card.ty {
                    price *= projectile_penalty;
                }
                if card.is_trigger {
                    price += 100.0;
                }
                let big_rand = rand::gen_range(-1, 4);
                price += big_rand as f32 * 10.0;
                price *= price_modifier;

                // round to nearest 5 and convert to u16
                let price = ((price + 2.5) / 5.0) as u16 * 5;

                row.push(Some((card, price)));
            }
            shop_cards.push(row);
        }
        self.shop = Some(Shop {
            cards: shop_cards,
            open: true,
        });
    }
    fn draw_inventory(&self, local_x: f32, local_y: f32, card_sheet: &Spritesheet) {
        if self.inventory_open {
            draw_square(SCREEN_WIDTH - INV_WIDTH, 0.0, INV_WIDTH, SCREEN_HEIGHT);
            self.text_engine
                .draw_text(SCREEN_WIDTH - INV_WIDTH + 2.0, 2.0, "cards", 1);
            for y in 0..self.inventory.len() {
                for x in 0..self.inventory[0].len() {
                    let tile_x = SCREEN_WIDTH - INV_WIDTH + 2.0 + x as f32 * CARD_SIZE;
                    let tile_y = 2.0 + y as f32 * CARD_SIZE + INV_MARGIN_TOP;
                    if let Some(card) = &self.inventory[y][x] {
                        card.draw(card_sheet, tile_x + 2.0, tile_y + 2.0);
                    } else {
                        draw_square(tile_x, tile_y, CARD_SIZE, CARD_SIZE);
                    }
                }
            }
        }
        let (handle_x, handle_y, flipped) = self.get_inv_handle_state();
        draw_img_button(
            card_sheet,
            handle_x,
            handle_y,
            local_x,
            local_y,
            32 * 3 + 1,
            flipped,
        );
    }
    fn handle_shop_input(&mut self, local_x: f32, local_y: f32) -> bool {
        let (handle_x, handle_y, _) = self.get_shop_handle_state();
        let Some(shop) = &mut self.shop else {
            return false;
        };

        if local_x > handle_x
            && local_x < handle_x + SPRITE_SIZE
            && local_y > handle_y
            && local_y < handle_y + SPRITE_SIZE
        {
            shop.open = !shop.open;
            return true;
        }
        if !shop.open {
            return false;
        }
        let shop_height = SHOP_PADDING + shop.cards.len() as f32 * SHOP_CARD_HEIGHT - 5.0;

        let shop_y = SCREEN_HEIGHT - shop_height;
        for y in 0..shop.cards.len() {
            for x in 0..shop.cards[0].len() {
                let tile_y = SHOP_PADDING + shop_y + 2.0 + y as f32 * SHOP_CARD_HEIGHT;
                let tile_x = 2.0 + x as f32 * SHOP_CARD_WIDTH;
                if self.cursor_card.is_none()
                    && local_x == local_x.clamp(tile_x, tile_x + CARD_SIZE)
                    && local_y == local_y.clamp(tile_y, tile_y + CARD_SIZE)
                    && shop.cards[y][x].is_some()
                {
                    let price = (shop.cards[y][x].as_ref()).map(|f| f.1).unwrap();
                    if self.gold >= price {
                        self.gold -= price;
                        let (card, _) = shop.cards[y][x].take().unwrap();
                        self.cursor_card = Some(card);
                        self.inventory_open = true;
                    }
                    return true;
                }
            }
        }

        false
    }
    fn draw_shop(&self, local_x: f32, local_y: f32, card_sheet: &Spritesheet) {
        let Some(shop) = &self.shop else {
            return;
        };
        let (handle_x, handle_y, flipped) = self.get_shop_handle_state();
        draw_img_button(
            card_sheet,
            handle_x,
            handle_y,
            local_x,
            local_y,
            32 * 3 + 1,
            flipped,
        );
        if !shop.open {
            return;
        }
        let shop_width = shop.cards[0].len() as f32 * SHOP_CARD_WIDTH + 4.0 - 7.0;
        let shop_height = SHOP_PADDING + shop.cards.len() as f32 * SHOP_CARD_HEIGHT - 5.0;
        let shop_x = 0.0;
        let shop_y = SCREEN_HEIGHT - shop_height;
        draw_square(shop_x, shop_y, shop_width, shop_height);
        self.text_engine
            .draw_text(shop_x + 2.0, shop_y + 2.0, "card shop", 1);
        for y in 0..shop.cards.len() {
            for x in 0..shop.cards[0].len() {
                let tile_y = SHOP_PADDING + shop_y + 2.0 + y as f32 * SHOP_CARD_HEIGHT;
                let tile_x = 2.0 + x as f32 * SHOP_CARD_WIDTH;
                if let Some((card, price)) = &shop.cards[y][x] {
                    self.text_engine
                        .draw_text(tile_x, tile_y - 5.0, &price.to_string(), 0);
                    card.draw(card_sheet, tile_x + 2.0, tile_y + 2.0);
                } else {
                    draw_square(tile_x, tile_y, CARD_SIZE, CARD_SIZE);
                }
            }
        }
        for y in 0..shop.cards.len() {
            for x in 0..shop.cards[0].len() {
                let tile_y = SHOP_PADDING + shop_y + 2.0 + y as f32 * SHOP_CARD_HEIGHT;
                let tile_x = 2.0 + x as f32 * SHOP_CARD_WIDTH;
                if let Some((card, _)) = &shop.cards[y][x] {
                    if local_x == local_x.clamp(tile_x, tile_x + CARD_SIZE)
                        && local_y == local_y.clamp(tile_y, tile_y + CARD_SIZE)
                    {
                        self.draw_card_info(local_x, local_y, card, card_sheet);
                    }
                }
            }
        }
    }
    /// Draws hover information of a card
    fn draw_card_info(
        &self,
        mut local_x: f32,
        mut local_y: f32,
        card: &Card,
        card_sheet: &Spritesheet,
    ) {
        if local_x > SCREEN_WIDTH / 2.0 {
            local_x -= CARD_INFO_WIDTH;
        }
        if local_y + CARD_INFO_HEIGHT + 4.0 > SCREEN_HEIGHT {
            local_y -= CARD_INFO_HEIGHT;
        }
        draw_square(local_x, local_y, CARD_INFO_WIDTH, CARD_INFO_HEIGHT);
        card.draw(card_sheet, local_x + 4.0, local_y + 4.0);
        let mut name = card.name.to_string();
        if card.is_trigger {
            name += " trigger"
        }
        self.text_engine
            .draw_text(local_x + 4.0 + CARD_SIZE, local_y + 3.0, &name, 1);
        self.text_engine
            .draw_text(local_x + 4.0 + CARD_SIZE, local_y + 8.0, card.desc, 0);
        let modifier_data = match &card.ty {
            CardType::Modifier(modifier_data) => modifier_data,
            CardType::Projectile(proj, _) => &proj.modifier_data,
            _ => {
                return;
            }
        };

        let mut index = 0.0;
        for (k, v) in modifier_data.iter() {
            self.text_engine.draw_text(
                local_x + 2.0,
                local_y + index * 5.0 + CARD_SIZE + 4.0,
                &format!("{k}:{v}"),
                2,
            );
            index += 1.0;
        }
    }
    fn get_inv_handle_state(&self) -> (f32, f32, bool) {
        if self.inventory_open {
            (
                (SCREEN_WIDTH - INV_WIDTH - SPRITE_SIZE),
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
    fn get_shop_handle_state(&self) -> (f32, f32, bool) {
        if let Some(shop) = &self.shop {
            let shop_width = shop.cards[0].len() as f32 * SHOP_CARD_WIDTH + 4.0 - 7.0;
            let shop_height = SHOP_PADDING + shop.cards.len() as f32 * SHOP_CARD_HEIGHT - 5.0;

            if shop.open {
                (
                    (shop_width),
                    (SCREEN_HEIGHT - shop_height + (shop_height / 2.0 - SPRITE_SIZE / 2.0)),
                    false,
                )
            } else {
                (
                    (0.0),
                    (SCREEN_HEIGHT - shop_height + (shop_height / 2.0 - SPRITE_SIZE / 2.0)),
                    true,
                )
            }
        } else {
            (0.0, 0.0, false)
        }
    }

    pub fn draw_ui(
        &self,
        local_x: f32,
        local_y: f32,
        card_sheet: &Spritesheet,
        selected_tower: Option<&Tower>,
    ) {
        if let Some(tower) = selected_tower {
            let width = (tower.card_slots.len() as f32 * CARD_SIZE + 4.0).max(68.0);
            draw_square(0.0, 7.0, width, CARD_SIZE + 4.0 + 5.0 * 3.0);
            let tile_y = 7.0 + 2.0;
            for (index, (k, v)) in [
                ("shoot delay", tower.shoot_delay),
                ("reload time", tower.recharge_speed),
            ]
            .iter()
            .enumerate()
            {
                self.text_engine.draw_text(
                    2.0,
                    tile_y + CARD_SIZE + 5.0 + 5.0 * index as f32,
                    &format!("{k}:{v}"),
                    2,
                );
            }
            for (index, card_slot) in tower.card_slots.iter().enumerate() {
                let tile_x = index as f32 * CARD_SIZE + 2.0;
                if let Some(card) = card_slot {
                    card.draw(card_sheet, tile_x + 2.0, tile_y + 2.0);
                } else {
                    draw_square(tile_x, tile_y, CARD_SIZE, CARD_SIZE);
                }
            }
        }
        self.draw_inventory(local_x, local_y, card_sheet);
        self.draw_shop(local_x, local_y, card_sheet);

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
            && local_x > SCREEN_WIDTH - INV_WIDTH + 2.0
            && local_x < SCREEN_WIDTH - 3.0
            && local_y > 2.0 + INV_MARGIN_TOP
        {
            let tile_x =
                (local_x as usize + INV_WIDTH_USIZE - SCREEN_WIDTH_USIZE - 2) / CARD_SIZE_USIZE;
            let tile_y = (local_y as usize - 2 - INV_MARGIN_TOP_USIZE) / CARD_SIZE_USIZE;

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
        let (handle_x, handle_y, _) = self.get_inv_handle_state();
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
        self.handle_shop_input(local_x, local_y)
    }
}

pub fn draw_square(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, COLOR_BROWN);
    draw_rectangle(x + 1.0, y + 1.0, w - 2.0, h - 2.0, COLOR_BEIGE);
}
pub fn draw_button(
    text_engine: &TextEngine,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    local_x: f32,
    local_y: f32,
    text: &str,
) -> bool {
    let hovered = local_x.clamp(x, x + w) == local_x && local_y.clamp(y, y + h) == local_y;
    let color_offset = if hovered { 0 } else { 2 };
    let bg = if hovered { WHITE } else { COLOR_YELLOW };

    draw_rectangle(x, y, w, h, COLOR_BROWN);
    draw_rectangle(x + 1.0, y + 1.0, w - 2.0, h - 2.0, bg);
    text_engine.draw_text(x + 2.0, y + 2.0, text, color_offset);
    hovered && is_mouse_button_pressed(MouseButton::Left)
}
pub fn draw_img_button(
    spritesheet: &Spritesheet,
    x: f32,
    y: f32,
    local_x: f32,
    local_y: f32,
    index: usize,
    flipped: bool,
) -> bool {
    let hovered = local_x.clamp(x, x + SPRITE_SIZE) == local_x
        && local_y.clamp(y, y + SPRITE_SIZE) == local_y;
    let sprite_offset = if hovered { 1 } else { 0 };

    spritesheet.draw_tile(x, y, index + sprite_offset, flipped, 0.0);
    hovered && is_mouse_button_pressed(MouseButton::Left)
}

pub fn draw_button_disabled(text_engine: &TextEngine, x: f32, y: f32, w: f32, h: f32, text: &str) {
    draw_rectangle(x, y, w, h, COLOR_BROWN);
    draw_rectangle(x + 1.0, y + 1.0, w - 2.0, h - 2.0, COLOR_BEIGE);
    text_engine.draw_text(x + 2.0, y + 2.0, text, 2);
}
