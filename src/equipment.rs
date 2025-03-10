use tcod::colors::*;

use crate::*;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
/// An object that can be equipped, yielding bonuses
pub struct Equipment {
    pub slot: Slot,
    pub equipped: bool,
    pub max_hp_bonus: i32,
    pub power_bonus: i32,
    pub defense_bonus: i32,
} 

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Slot {
    LeftHand,
    RightHand,
    Head,
}

impl Object {
    /// Equip object and show message
    pub fn equip(&mut self, messages: &mut Messages) {
        if self.item.is_none() {
            messages.add(format!("Cannot equip {:?} because it's not an Item.", self), RED.into());
            return;
        }

        if let Some(ref mut equipment) = self.equipment {
            if !equipment.equipped {
                equipment.equipped = true;
                messages.add(format!("Equipped {} on {}.", self.name, equipment.slot), LIGHT_GREEN.into());
            }
        } else {
            messages.add(format!("Cannot equip {:?} because it's not Equipment.", self), RED.into());
        }
    }

    pub fn unequip(&mut self, messages: &mut Messages) {
        if self.item.is_none() {
            messages.add(format!("Cannot unequip {:?} because it's not an Item.", self), RED.into());
            return;
        }
        
        if let Some(ref mut equipment) = self.equipment {
            if equipment.equipped {
                equipment.equipped = false;
                messages.add(format!("Unequiped {} from {}.", self.name, equipment.slot), LIGHT_YELLOW.into());
            }
        } else {
            messages.add(format!("Cannot unequip {:?} because it's not Equipment", self), RED.into());
        }
    }

    /// Returns a list of all equipped items
    pub fn get_all_equipped(&self, game: &Game) -> Vec<Equipment> {
        if self.name == "player" {
            game.inventory
                .iter()
                .filter(|item| item.equipment.map_or(false, |e| e.equipped))
                .map(|item| item.equipment.unwrap())
                .collect()
        } else {
            vec![]  // Other objects have no equipment
        }
    }
}


//-----------------------------//
// EQUIPMENT BONUSES           //
//-----------------------------//
impl Object {
    pub fn power(&self, game: &Game) -> i32 {
        let base_power = self.fighter.map_or(0, |f| f.base_power);
        let bonus: i32 = self.get_all_equipped(game).iter().map(|e| e.power_bonus).sum();
        base_power + bonus
    }

    pub fn defense(&self, game: &Game) -> i32 {
        let base_defense = self.fighter.map_or(0, |f| f.base_defense);
        let bonus: i32 = self
            .get_all_equipped(game)
            .iter()
            .map(|e| e.defense_bonus)
            .sum();
        base_defense + bonus
    }
}


pub fn toggle_equipment(inventory_id: usize, _tcod: &mut Tcod, game: &mut Game, _objects: &mut [Object]) -> UseResult {
    let equipment = match game.inventory[inventory_id].equipment {
        Some(equipment) => equipment,
        None => return UseResult::Cancelled,
    };

    // If slot is already being used, unequip it first
    if let Some(current) = get_equipped_in_slot(equipment.slot, &game.inventory) {
        game.inventory[current].unequip(&mut game.messages);
    }

    if equipment.equipped { game.inventory[inventory_id].unequip(&mut game.messages); }
    else { game.inventory[inventory_id].equip(&mut game.messages); }

    UseResult::UsedAndKept
}

pub fn get_equipped_in_slot(slot: Slot, inventory: &[Object]) -> Option<usize> {
    for (inventory_id, item) in inventory.iter().enumerate() {
        if item.equipment.as_ref().map_or(false, |e| e.equipped && e.slot == slot) {
            return Some(inventory_id);
        }
    }
    None
}

impl std::fmt::Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Slot::LeftHand => write!(f, "left hand"),
            Slot::RightHand => write!(f, "right hand"),
            Slot::Head => write!(f, "head"),
        }
    }
}