use tcod::colors::*;

use crate::*;

/// LEVELING UP
pub struct Transition {
    pub level: u32,
    pub value: u32,
}

/// Returns a value that depends on level
/// The table specifies what value occurs at each level
pub fn from_dungeon_level(table: &[Transition], level: u32) -> u32 {
    table
        .iter()
        .rev()
        .find(|t| level >= t.level)
        .map_or(0, |t| t.value)
}

pub fn level_up(tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) {
    let player = &mut objects[PLAYER];
    let level_up_xp = LEVEL_UP_BASE + player.level as i32 * LEVEL_UP_FACTOR;

    // Check if player has enough xp to level up
    if player.fighter.as_ref().map_or(0, |f| f.xp) >= level_up_xp {
        // Level up!
        player.level += 1;
        game.messages.add(format!("You leveled up! You are now level {}", player.level), YELLOW.into());

        let fighter = player.fighter.as_mut().unwrap();
        let mut choice = None;

        while choice.is_none() {
            // Keep asking until a choice is made
            choice = menu(
                "Level up! Choose a stat to raise:\n",
                &[
                    format!("Vitality (+20 HP, from {})", fighter.max_hp),
                    format!("Strength (+1 Attack, from {})", fighter.power),
                    format!("Agility (+1 Defense, from {})", fighter.defense),
                ],
                LEVEL_SCREEN_WIDTH,
                &mut tcod.root,
            );
        }
        fighter.xp -= level_up_xp;
        match choice.unwrap() {
            0 => {
                fighter.max_hp += 20;
            }
            1 => {
                fighter.power += 1;
            }
            2 => {
                fighter.defense += 1;
            }
            _ => unreachable!(),
        }
    }
}