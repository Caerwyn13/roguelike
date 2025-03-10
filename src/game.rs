use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

use tcod::{
    input::Event,
    colors::*,
    input
};

use crate::*;

pub fn main_menu(tcod: &mut Tcod) {
    let img = tcod::image::Image::from_file("assets/main_menu.png")
        .ok()
        .expect("Menu image not found");
    

    while !tcod.root.window_closed() {
        // Show background image
        tcod::image::blit_2x(&img, (0, 0), (-1, -1), &mut tcod.root, (0, 0));

        // Title and credits
        tcod.root.set_default_foreground(LIGHT_YELLOW.into());
        tcod.root.print_ex(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2 - 4, BackgroundFlag::None, TextAlignment::Center, "TOMBS OF THE FALLEN");
        tcod.root.print_ex(SCREEN_WIDTH / 2, SCREEN_HEIGHT - 2, BackgroundFlag::None, TextAlignment::Center, "By Caerwyn S-R");

        // Show options and wait for choice
        let choices = &["New Game", "Continue", "Quit"];
        let choice = menu("", choices, 24, &mut tcod.root);

        match choice {
            Some(0) => {
                // Confirm if the player really wants to start a new game
                let confirm_choices = &["Yes", "No"];
                let confirm = menu("\nStarting a new game will erase current progress. Are you sure?", confirm_choices, 36, &mut tcod.root);

                if confirm == Some(0) {
                    // Proceed with new game
                    let (mut game, mut objects) = new_game(tcod);
                    play_game(tcod, &mut game, &mut objects);
                }
            }
            Some(1) => {
                // Load game
                match load_game() {
                    Ok((mut game, mut objects)) => {
                        initialise_fov(tcod, &game.map);
                        play_game(tcod, &mut game, &mut objects);
                    }
                    Err(_e) => {
                        msgbox("\nNo saved game to load :(\n", 24, &mut tcod.root);
                    }
                }
            }
            Some(2) => {
                // Quit
                break;
            }

            _ => {}
        }
    }
}

pub fn new_game(tcod: &mut Tcod) -> (Game, Vec<Object>) {
    // Create objects
    let mut player = Object::new(0, 0, '@', WHITE.into(), "player", true);
    player.alive = true;
    player.fighter = Some(Fighter {
        base_max_hp: 100,
        hp: 100,
        base_defense: 1,
        base_power: 2,
        xp: 0,
        on_death: DeathCallback::Player,
    });

    let mut objects = vec![player];
    let mut game = Game { map: make_map(&mut objects, 1), messages: Messages::new(), inventory: vec![], dungeon_level: 1 };

    initialise_fov(tcod, &game.map);

    // Give the player a starting dagger
    let mut dagger = Object::new(0, 0, '-', SKY.into(), "Dagger", false);
    dagger.item = Some(Item::Sword);
    dagger.equipment = Some(Equipment {
        equipped: true,
        slot: Slot::LeftHand,
        max_hp_bonus: 0,
        defense_bonus: 0,
        power_bonus: 2,
    });
    game.inventory.push(dagger);

    // Welcome message!
    game.messages.add(
        "Welcome stranger! Prepare to perish in the Tombs of the Fallen.",
        RED.into(),    
    );

    (game, objects)
}

pub fn play_game(tcod: &mut Tcod, game: &mut Game, objects: &mut Vec<Object>) {
    // Force FOV "recompute" first time through game loop
    let mut previous_player_position = (-1, -1);
    while !tcod.root.window_closed() {
        tcod.con.clear();

        // Check for mouse
        match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
            Some((_, Event::Mouse(m))) => tcod.mouse = m,
            Some((_, Event::Key(k))) => tcod.key = k,
            _ => tcod.key = Default::default(),
        }

        let fov_recompute = previous_player_position != (objects[PLAYER].x, objects[PLAYER].y);
        render_all(tcod, game, &objects, fov_recompute);
        
        tcod.root.flush();
        level_up(tcod, game, objects);

        // Handle keys and exit if needed
        previous_player_position = objects[PLAYER].pos();

        let player_action = handle_keys(tcod, game, objects);
        if player_action == PlayerAction::Exit {
            save_game(game, objects).unwrap();
            break; 
        }

        // Let monsters take their turn
        if objects[PLAYER].alive && player_action != PlayerAction::DidntTakeTurn {
            for id in 0..objects.len() {
                if objects[id].ai.is_some() {
                    ai_take_turn(id, tcod, game, objects);
                }
            }
        }
    }
}

fn save_game(game: &Game, objects: &[Object]) -> Result<(), Box<dyn Error>> {
    let save_data = serde_json::to_string(&(game, objects))?;
    let mut file = File::create("savegame")?;
    file.write_all(save_data.as_bytes())?;
    Ok(())
}

fn load_game() -> Result<(Game, Vec<Object>), Box<dyn Error>> {
    let mut json_save_state = String::new();
    let mut file = File::open("savegame")?;
    file.read_to_string(&mut json_save_state)?;
    let result = serde_json::from_str::<(Game, Vec<Object>)>(&json_save_state)?;
    Ok(result)
}

pub fn next_level(tcod: &mut Tcod, game: &mut Game, objects: &mut Vec<Object>) {
    game.messages.add("You take a moment to rest, and recover your strength.", VIOLET.into());
    
    let heal_hp = objects[PLAYER].max_hp(game) / 2;
    objects[PLAYER].heal(heal_hp, game);

    game.messages.add("You descend deeper into the dungeon...", RED.into());

    game.dungeon_level += 1;
    game.map = make_map(objects, game.dungeon_level);
    initialise_fov(tcod, &game.map);
}


fn initialise_fov(tcod: &mut Tcod, map: &Map) {
    // Populate FOV map according to the generated map
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            tcod.fov.set(
                x,
                y,
                !map[x as usize][y as usize].block_sight,
                !map[x as usize][y as usize].blocked,
            );
        }
    }

    tcod.con.clear();
}