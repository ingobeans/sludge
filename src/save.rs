use std::path::PathBuf;

use bincode::{decode_from_std_read, encode_into_std_write, Decode, Encode};
use macroquad::math::Vec2;

use crate::cards::get_cards;
use crate::map::Map;
use crate::tower::get_towers;
use crate::ui::TextEngine;
use crate::Sludge;
use crate::{consts::*, ui};

fn get_save_path() -> PathBuf {
    let exe = std::env::current_exe().expect("couldn't get path to executable!");
    let dir = exe.parent().unwrap();
    dir.join("save.sldg")
}
pub fn save_exists() -> bool {
    get_save_path().exists()
}
#[derive(Debug, PartialEq, Clone, Decode, Encode)]
pub struct TowerSaveData {
    x: f32,
    y: f32,
    direction: f32,
    slots: [Option<u8>; TOWER_MAX_SPELLS],
}
#[derive(Debug, PartialEq, Clone, Decode, Encode)]
pub struct SaveData {
    pub lives: u8,
    pub gold: u16,
    pub round_index: u8,
    pub map_index: u8,
    pub shop_items:
        [Option<(u8, u16)>; DEFAULT_SHOP_SLOTS_HORIZONTAL * DEFAULT_SHOP_SLOTS_VERTICAL],
    pub towers: [Option<TowerSaveData>; 4],
}
impl SaveData {
    pub fn create(sludge: &Sludge) -> Self {
        let all_cards = get_cards();
        let mut shop_items = sludge.ui_manager.shop.as_ref().unwrap().cards.clone();

        let shop_items: [Option<(u8, u16)>; _] = std::array::from_fn(|index| {
            shop_items[index / DEFAULT_SHOP_SLOTS_HORIZONTAL][index % DEFAULT_SHOP_SLOTS_HORIZONTAL]
                .take()
                .map(|(card, price)| {
                    (
                        all_cards.iter().position(|f| f == &card).unwrap() as u8,
                        price,
                    )
                })
        });
        let all_towers = get_towers([(0, 0); 4]);
        let mut towers = std::array::from_fn(|_| None);
        for tower in sludge.towers.iter() {
            let mut slots = std::array::from_fn(|_| None);
            for (index, slot) in tower.card_slots.iter().enumerate() {
                slots[index] = slot
                    .as_ref()
                    .map(|card| (all_cards.iter().position(|f| f == card).unwrap() as u8))
            }
            let index = all_towers.iter().position(|f| f == tower).unwrap();
            towers[index] = Some(TowerSaveData {
                x: tower.x,
                y: tower.y,
                direction: tower.direction.to_angle(),
                slots,
            });
        }
        Self {
            lives: sludge.lives,
            gold: sludge.ui_manager.gold,
            round_index: sludge.round_manager.round as u8,
            map_index: sludge.map_index as u8,
            shop_items,
            towers,
        }
    }
    pub fn load(&self, maps: &[Map], text_engine: TextEngine) -> Sludge {
        let all_cards = get_cards();
        let mut new = Sludge::new(
            maps[self.map_index as usize].clone(),
            self.map_index as usize,
            text_engine,
            false,
        );
        new.lives = self.lives;
        new.ui_manager.gold = self.gold;
        new.round_manager.round = self.round_index as usize;
        let mut cards = Vec::new();
        for (index, item) in self.shop_items.into_iter().enumerate() {
            let y = index / DEFAULT_SHOP_SLOTS_HORIZONTAL;
            if y >= cards.len() {
                cards.push(Vec::new());
            }
            let item = item.map(|(index, price)| (all_cards[index as usize].clone(), price));
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
                    let card = card_data.map(|f| all_cards[f as usize].clone());
                    tower.card_slots[card_index] = card;
                }
                towers.push(tower);
            }
        }

        new.ui_manager.shop = Some(shop);
        new.towers = towers;
        new
    }
}

pub fn write_save(data: SaveData) {
    let binary = get_save_path();
    let Ok(mut binary) = std::fs::File::create(binary) else {
        return;
    };
    let _ = encode_into_std_write(data, &mut binary, bincode::config::standard());
}
pub fn read_save() -> Option<SaveData> {
    if !save_exists() {
        return None;
    }
    let binary = get_save_path();
    let mut binary = std::fs::File::open(binary).ok()?;
    let data = decode_from_std_read(&mut binary, bincode::config::standard()).ok()?;
    Some(data)
}

#[cfg(test)]
mod tests {

    use crate::save::{read_save, write_save, SaveData};

    #[test]
    fn encode_decode() {
        let old_data = SaveData {
            lives: 88,
            round_index: 67,
            gold: 1455,
            shop_items: std::array::from_fn(|_| None),
            map_index: 1,
            towers: std::array::from_fn(|_| None),
        };
        write_save(old_data.clone());
        let data = read_save().unwrap();
        println!("{:?}", data);
        assert_eq!(data, old_data)
    }
}
