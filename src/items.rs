use tcod::colors::*;

use crate::*;

const HEAL_AMOUNT: i32 = 40;

const LIGHTNING_DAMAGE: i32 = 20;
const LIGHTNING_RANGE: i32 = 5;

const CONFUSION_RANGE: i32 = 8;
const CONFUSION_NUM_TURNS: i32 = 10;

const FIREBALL_DAMAGE: i32 = 25;
const FIREBALL_RADIUS: i32 = 3;


#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Item {
    Heal,
    Lightning,
    Confuse,
    Fireball,
    Sword,
    Shield,
}

/// Add to player's inventory and remove from map
pub fn pick_item_up(object_id: usize, game: &mut Game, objects: &mut Vec<Object>) {
    if game.inventory.len() >= 26 {
        game.messages.add(format!("Your inventory is full, cannot pick up {}", objects[object_id].name), RED.into());
    } else {
        let item = objects.swap_remove(object_id);
        game.messages.add(format!("You picked up {}", item.name), GREEN.into());
        let index = game.inventory.len();
        let slot = item.equipment.map(|e| e.slot);
        game.inventory.push(item);

        // Automatically equip, if the current slot is unused
        if let Some(slot) = slot {
            if get_equipped_in_slot(slot, &game.inventory).is_none() {
                game.inventory[index].equip(&mut game.messages);
            }
        }
    }
}

pub enum UseResult {
    UsedUp,
    UsedAndKept,
    Cancelled,
}

pub fn use_item(inventory_id: usize, tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) {
    use Item::*;

    // Call relative use function if defined
    if let Some(item) = game.inventory[inventory_id].item {
        let on_use: fn(usize, &mut Tcod, &mut Game, &mut [Object]) -> UseResult = match item {
            Heal => cast_heal,
            Lightning => cast_lightning,
            Confuse => cast_confuse,
            Fireball => cast_fireball,
            Sword => toggle_equipment,
            Shield => toggle_equipment,
        };
        match on_use(inventory_id, tcod, game, objects) {
            UseResult::UsedUp => {
                // Destroy after use
                game.inventory.remove(inventory_id);
            }
            UseResult::UsedAndKept => {}    // No need for action
            UseResult::Cancelled => {
                game.messages.add("Cancelled", WHITE.into());
            }
        }
    } else {
        game.messages.add(format!("The {} cannot be used", game.inventory[inventory_id].name), WHITE.into());
    }
}

pub fn drop_item(inventory_id: usize, game: &mut Game, objects: &mut Vec<Object>) {
    let mut item = game.inventory.remove(inventory_id);
    if item.equipment.is_some() { item.unequip(&mut game.messages); }   // Unequip item when dropped

    item.set_pos(objects[PLAYER].x, objects[PLAYER].y);

    game.messages.add(format!("You dropped your {}.", item.name), YELLOW.into());
    objects.push(item);
}



//-----------------------------//
// ITEM FUNCTIONS              //
//-----------------------------//
fn cast_heal(_inventory_id: usize, _tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) -> UseResult {
    // Heal the player
    let player = &mut objects[PLAYER];
    if let Some(fighter) = player.fighter {
        if fighter.hp == fighter.base_max_hp { 
            game.messages.add("You are already at full health!", RED.into());
            return UseResult::Cancelled;
        }
        game.messages.add("Your wounds are healed", LIGHT_VIOLET.into());
        player.heal(HEAL_AMOUNT, game);
        return UseResult::UsedUp;
    }
    UseResult::Cancelled
}

fn cast_lightning(_inventory_id: usize, tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) -> UseResult {
    // Find closest enemy (inside max range)
    let monster_id = closest_monster(tcod, objects, LIGHTNING_RANGE);

    if let Some(monster_id) = monster_id {
        // Zap monster
        game.messages.add(format!("A lightning bolt strikes the {} with a loud thunder! \n It takes {} damage", objects[monster_id].name, LIGHTNING_DAMAGE), LIGHT_BLUE.into());
        if let Some(xp) = objects[monster_id].take_damage(LIGHTNING_DAMAGE, game) {
            objects[PLAYER].fighter.as_mut().unwrap().xp += xp;
        }
        UseResult::UsedUp
    } else {
        // No enemy found within range
        game.messages.add("No enemy is close enough to strike.", RED.into());
        UseResult::Cancelled
    }
}

fn cast_confuse(_inventory_id: usize, tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) -> UseResult {
    game.messages.add( "Left-click on an enemy to confuse it, or Right-click to cancel.", LIGHT_CYAN.into());
    let monster_id = target_monster(tcod, game, objects, Some(CONFUSION_RANGE as f32));

    if let Some(monster_id) = monster_id {
        let old_ai = objects[monster_id].ai.take().unwrap_or(Ai::Basic);

        // Replace AI with confused one
        objects[monster_id].ai = Some(Ai::Confused { previous_ai: Box::new(old_ai), num_turns: CONFUSION_NUM_TURNS });
        game.messages.add(format!("The eyes of the {} look vacant, as they start to stumble around", objects[monster_id].name), LIGHT_GREEN.into());
        UseResult::UsedUp
    } else {
        // No enemy found in range
        game.messages.add("No enemy is close enough to confuse", RED.into());
        UseResult::Cancelled
    }
}

fn cast_fireball(_inventory_id: usize, tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) -> UseResult {
    // Ask the player for a target tile
    game.messages.add("Left-click on a target tile for the fireball, or Right-click to cancel.", LIGHT_CYAN.into());
    let (x, y) = match target_tile(tcod, game, objects, None) {
        Some(tile_pos) => tile_pos,
        None => return UseResult::Cancelled,
    };
    game.messages.add(format!("The fireball explodes, burning everything within {} tiles!", FIREBALL_RADIUS), ORANGE.into());

    let mut xp_to_gain = 0;

    for (id, obj) in objects.iter_mut().enumerate() {
        if obj.distance(x, y) <= FIREBALL_RADIUS as f32 && obj.fighter.is_some() && id != PLAYER {
            game.messages.add(
                format!("The {} gets burned for {} hit points.", obj.name, FIREBALL_DAMAGE),
                ORANGE.into(),
            );

            if let Some(xp) = obj.take_damage(FIREBALL_DAMAGE, game) {
                xp_to_gain += xp; // Only add XP if it's a valid enemy
            }
        }
    }
    // Reward the player with accumulated XP
    objects[PLAYER].fighter.as_mut().unwrap().xp += xp_to_gain;

    UseResult::UsedUp
}