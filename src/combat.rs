use tcod::colors::*;

use crate::*;

impl Object {
    pub fn attack(&mut self, target: &mut Object, game: &mut Game) {
        // A simple formula for attack damage
        let damage = self.fighter.map_or(0, |f| f.base_power) - target.fighter.map_or(0, |f| f.base_defense);

        if damage > 0 {
            // Take damage
            game.messages.add(format!("{} attacks {} for {} damage.", self.name, target.name, damage), WHITE.into());
            
            if let Some(xp) = target.take_damage(damage, game) {
                // Give XP to player
                self.fighter.as_mut().unwrap().xp += xp;
            }
        } else {
            game.messages.add(format!("{} attacks {} but it has no effect!", self.name, target.name), WHITE.into());
        }
    }

    pub fn take_damage(&mut self, damage: i32, game: &mut Game) -> Option<i32> {
        // Apply damage if possible
        if let Some(fighter) = self.fighter.as_mut() {
            if damage > 0 { fighter.hp -= damage; }
        }

        // Check for death, call function if needed
        if let Some(fighter) = self.fighter {
            if fighter.hp <= 0 {
                self.alive = false;
                fighter.on_death.callback(self, game);
                return Some(fighter.xp);
            }
        }
        None
    }

    /// Heal by the given amount, without going over the maximum
    pub fn heal(&mut self, amount: i32, game: &Game) {  
        let max_hp = self.max_hp(game);  
        if let Some(ref mut fighter) = self.fighter {
            fighter.hp += amount;
            if fighter.hp > max_hp {  
                fighter.hp = max_hp;  
            }
        }
    }
}

pub fn player_move_or_attack(dx: i32, dy: i32, game: &mut Game, objects: &mut[Object]) {
    // Coords to move/attack to
    let x= objects[PLAYER].x + dx;
    let y= objects[PLAYER].y + dy;

    // Try to find an attackable object there
    let target_id = objects.iter().position(|object| object.fighter.is_some() && object.pos() == (x, y));

    // Attack if target_id != None
    match target_id {
        Some(target_id) => {
            let (player, target) = mut_two(PLAYER, target_id, objects);
            player.attack(target, game);
        }
        None => {
            move_by(PLAYER, dx, dy, &game.map, objects);
        }
    }
}

pub fn closest_monster(tcod: &Tcod, objects: &[Object], max_range: i32) -> Option<usize> {
    let mut closest_enemy = None;
    let mut closest_dist = (max_range + 1) as f32;  // Start with slightly more than max range

    for (id, object) in objects.iter().enumerate() {
        if (id != PLAYER) && object.fighter.is_some() && object.ai.is_some() && tcod.fov.is_in_fov(object.x, object.y) {
            // Calculate distance to object
            let dist = objects[PLAYER].distance_to(object);
            if dist < closest_dist {
                // Current enemy is closer
                closest_enemy = Some(id);
                closest_dist = dist;
            }
        }
    }

    closest_enemy
}

pub fn target_monster(tcod: &mut Tcod, game: &mut Game, objects: &[Object], max_range: Option<f32>) -> Option<usize> {
    loop {
        match target_tile(tcod, game, objects, max_range) {
            Some((x, y)) => {
                // Return the first clicked monster, otherwise continue looping
                for (id, obj) in objects.iter().enumerate() {
                    if obj.pos() == (x, y) && obj.fighter.is_some() && id != PLAYER {
                        return Some(id);
                    }
                }
            }
            None => return None,
        }
    }
}