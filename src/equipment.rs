use tcod::colors::*;

use crate::*;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
/// An object that can be equipped, yielding bonuses
pub struct Equipment {
    pub slot: Slot,
    pub equipped: bool,
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
                messages.add(format!("Equipped {} on {:?}.", self.name, equipment.slot), LIGHT_GREEN.into());
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
                messages.add(format!("Unequiped {} from {:?}.", self.name, equipment.slot), LIGHT_YELLOW.into());
            }
        } else {
            messages.add(format!("Cannot unequip {:?} because it's not Equipment", self), RED.into());
        }
    }
}