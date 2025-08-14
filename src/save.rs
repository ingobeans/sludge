#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use bincode::{Decode, Encode};
use macroquad::math::Vec2;

#[cfg(not(target_arch = "wasm32"))]
use bincode::{decode_from_std_read, encode_into_std_write};

#[cfg(target_arch = "wasm32")]
use base64::{prelude::BASE64_STANDARD, Engine};
#[cfg(target_arch = "wasm32")]
use bincode::{decode_from_slice, encode_to_vec};

use crate::cards::{get_cards, library, Card};
use crate::map::Map;
use crate::tower::get_towers;
use crate::ui::TextEngine;
use crate::Sludge;
use crate::{consts::*, ui};

#[cfg(not(target_arch = "wasm32"))]
fn get_save_path() -> PathBuf {
    let exe = std::env::current_exe().expect("couldn't get path to executable!");
    let dir = exe.parent().unwrap();
    dir.join("save.sldg")
}
pub fn save_exists() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        get_save_path().exists()
    }
    #[cfg(target_arch = "wasm32")]
    {
        quad_storage::STORAGE.lock().unwrap().get("save").is_some()
    }
}
pub fn remove_save() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if save_exists() {
            let _ = std::fs::remove_file(get_save_path());
        }
    }
    #[cfg(target_arch = "wasm32")]
    quad_storage::STORAGE.lock().unwrap().clear();
}
#[derive(Debug, PartialEq, Clone, Decode, Encode)]
pub struct TowerSaveData {
    x: f32,
    y: f32,
    direction: f32,
    slots: [Option<u8>; TOWER_MAX_SLOTS],
}
#[derive(Debug, PartialEq, Clone, Decode, Encode)]
pub struct SaveData {
    pub seed: u64,
    pub lives: u8,
    pub gold: u16,
    pub round_index: u8,
    pub map_index: u8,
    pub shop_items:
        [Option<(u8, u16)>; DEFAULT_SHOP_SLOTS_HORIZONTAL * DEFAULT_SHOP_SLOTS_VERTICAL],
    pub towers: [Option<TowerSaveData>; 4],
    pub inventory: [Option<u8>; INV_SLOTS_HORIZONTAL * INV_SLOTS_VERTICAL],
}
fn actualize_virtual_card(mut card: u8, cards: &[Card]) -> Card {
    let mut trigger = false;
    let cards_len = cards.len() as u8;
    if card > cards_len {
        trigger = true;
        card -= cards_len;
    }
    let card = cards[card as usize].clone();
    if trigger {
        library::as_trigger(card)
    } else {
        card
    }
}
fn virtualize_card(card: &Card, cards: &[Card]) -> u8 {
    let mut index = cards.iter().position(|f| f == card).unwrap() as u8;
    let cards_len = cards.len() as u8;
    if card.is_trigger {
        index += cards_len;
    }
    index
}
impl SaveData {
    pub fn create(sludge: &Sludge) -> Self {
        let all_cards = get_cards();
        let mut shop_items = sludge.ui_manager.shop.as_ref().unwrap().cards.clone();

        let shop_items: [Option<(u8, u16)>; _] = std::array::from_fn(|index| {
            shop_items[index / DEFAULT_SHOP_SLOTS_HORIZONTAL][index % DEFAULT_SHOP_SLOTS_HORIZONTAL]
                .take()
                .map(|(card, price)| (virtualize_card(&card, &all_cards), price))
        });
        let all_towers = get_towers([(0, 0); 4]);
        let mut towers = std::array::from_fn(|_| None);
        for tower in sludge.towers.iter() {
            let mut slots = std::array::from_fn(|_| None);
            for (index, slot) in tower.card_slots.iter().enumerate() {
                slots[index] = slot.as_ref().map(|card| virtualize_card(card, &all_cards))
            }
            let index = all_towers.iter().position(|f| f == tower).unwrap();
            towers[index] = Some(TowerSaveData {
                x: tower.x,
                y: tower.y,
                direction: tower.direction.to_angle(),
                slots,
            });
        }
        let inventory = std::array::from_fn(|index| {
            sludge.ui_manager.inventory[index / INV_SLOTS_HORIZONTAL][index % INV_SLOTS_HORIZONTAL]
                .as_ref()
                .map(|f| virtualize_card(f, &all_cards))
        });
        Self {
            seed: sludge.seed,
            lives: sludge.lives,
            gold: sludge.ui_manager.gold,
            round_index: sludge.round_manager.round as u8,
            map_index: sludge.map_index as u8,
            shop_items,
            towers,
            inventory,
        }
    }
    pub async fn load(&self, maps: &[Map], text_engine: TextEngine) -> Sludge {
        let all_cards = get_cards();
        let mut new = Sludge::new(
            maps[self.map_index as usize].clone(),
            self.map_index as usize,
            text_engine,
            false,
            self.seed,
        )
        .await;
        new.lives = self.lives;
        new.ui_manager.gold = self.gold;
        new.round_manager.round = self.round_index as usize;
        let mut cards = Vec::new();
        for (index, item) in self.shop_items.into_iter().enumerate() {
            let y = index / DEFAULT_SHOP_SLOTS_HORIZONTAL;
            if y >= cards.len() {
                cards.push(Vec::new());
            }
            let item =
                item.map(|(index, price)| (actualize_virtual_card(index, &all_cards), price));
            cards.last_mut().unwrap().push(item);
        }
        let shop = ui::Shop { cards, open: false };
        let mut towers = Vec::new();
        let all_towers = get_towers([(0, 0); 4]);
        for (index, tower_data) in self.towers.iter().enumerate() {
            if let Some(tower_data) = tower_data {
                let mut tower = all_towers[index].clone();
                tower.x = tower_data.x;
                tower.y = tower_data.y;
                tower.direction = Vec2::from_angle(tower_data.direction);
                for (card_index, card_data) in tower_data.slots.iter().enumerate() {
                    if card_index >= tower.card_slots.len() {
                        break;
                    }
                    let card = card_data.map(|f| actualize_virtual_card(f, &all_cards));
                    tower.card_slots[card_index] = card;
                }
                towers.push(tower);
            }
        }
        let mut inventory = Vec::new();
        for y in 0..INV_SLOTS_VERTICAL {
            inventory.push(std::array::from_fn(|x| {
                self.inventory[y * INV_SLOTS_HORIZONTAL + x]
                    .map(|card| actualize_virtual_card(card, &all_cards))
            }));
        }

        new.ui_manager.shop = Some(shop);
        new.towers = towers;
        new.ui_manager.inventory = inventory;
        new
    }
}

pub fn write_save(data: SaveData) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let binary = get_save_path();
        let Ok(mut binary) = std::fs::File::create(binary) else {
            return;
        };
        let _ = encode_into_std_write(data, &mut binary, bincode::config::standard());
    }

    #[cfg(target_arch = "wasm32")]
    {
        let Ok(data) = encode_to_vec(data, bincode::config::standard()) else {
            return;
        };
        let text = BASE64_STANDARD.encode(&data);
        let _ = quad_storage::STORAGE.lock().unwrap().set("save", &text);
    }
}
pub fn read_save() -> Option<SaveData> {
    if !save_exists() {
        return None;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let binary = get_save_path();
        let mut binary = std::fs::File::open(binary).ok()?;
        let data = decode_from_std_read(&mut binary, bincode::config::standard()).ok()?;
        Some(data)
    }

    #[cfg(target_arch = "wasm32")]
    {
        let data = quad_storage::STORAGE.lock().unwrap().get("save")?;
        let data = BASE64_STANDARD.decode(&data).ok()?;
        let (data, _) = decode_from_slice(&data, bincode::config::standard()).ok()?;
        Some(data)
    }
}
