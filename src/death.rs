use tcod::colors::*;

use crate::*;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DeathCallback {
    Player,
    Monster,
}

impl DeathCallback {
    pub fn callback(self, object: &mut Object, game: &mut Game) {
        use DeathCallback::*;
        let callback = match self {
            Player => player_death,
            Monster => monster_death,
        };
        callback(object, game);
    }
}

fn player_death(player: &mut Object, game: &mut Game) {
    // Game ended
    game.messages.add("You Died!", RED.into());

    // For added effect, turn player into corpse
    player.char = '%';
    player.colour = DARK_RED.into();
}

fn monster_death(monster: &mut Object, game: &mut Game) {
    // Transform into non-blocking, non-attacking, non-moving corpse
    game.messages.add(
        format!(
            "{} is dead! You gain {} experience points.",
            monster.name,
            monster.fighter.unwrap().xp
        ),
        ORANGE.into(),
    );
    monster.char = '%';
    monster.colour = DARKER_RED.into();
    monster.blocks = false;
    monster.fighter = None;
    monster.ai = None;
    monster.name = format!("remains of {}", monster.name);
} 