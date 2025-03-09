use std::cmp;
use rand::Rng;
use tcod::{
    console::*,
    input::{Key, Mouse},
    map::{FovAlgorithm, Map as FovMap},
};

use crate::{
    ai::*,
    colour::*,
    combat::*,
    death::*,
    equipment::*,
    game::*,
    gui::*,
    items::*,
    leveling::*,
    map::*,
    message::*,
    movement::*,
    object::*,
    tile::*,
};

mod ai;
mod colour;
mod combat;
mod death;
mod equipment;
mod game;
mod gui;
mod items;
mod leveling;
mod map;
mod message;
mod movement;
mod object;
mod tile;



// Size of the screen
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;


//-----------------------------//
// GUI CONSTANTS               //
//-----------------------------//
const BAR_WIDTH: i32 = 20;
const PANEL_HEIGHT: i32 = 7;
const PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;

const LEVEL_SCREEN_WIDTH: i32 = 40;
const CHARACTER_SCREEN_WIDTH: i32 = 30;
const INVENTORY_WIDTH: i32 = 50;

const MSG_X: i32 = BAR_WIDTH + 2;
const MSG_WIDTH: i32 = SCREEN_WIDTH - BAR_WIDTH - 2;
const MSG_HEIGHT: usize = PANEL_HEIGHT as usize - 1;


//-----------------------------//
// MAP CONSTANTS               //
//-----------------------------//
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 43;     // Space for panels with stats
const COLOUR_DARK_WALL: SerializableColour = SerializableColour { r: 0, g: 0, b: 100 };
const COLOUR_LIGHT_WALL: SerializableColour = SerializableColour { r: 130, g: 110, b: 50 };
const COLOUR_DARK_GROUND: SerializableColour = SerializableColour { r: 50, g: 50, b: 150 };
const COLOUR_LIGHT_GROUND: SerializableColour = SerializableColour { r: 200, g: 180, b: 50, };

const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MIN_ROOMS: i32 = 5;
const MAX_ROOMS: i32 = 30;


//-----------------------------//
// PLAYER CONSTANTS            //
//-----------------------------//
const PLAYER: usize = 0;            // Player will always be first object in array
const LEVEL_UP_BASE: i32 = 200;
const LEVEL_UP_FACTOR: i32 = 150;


//-----------------------------//
// FOV CONSTANTS               //
//-----------------------------//
const FOV_ALGORITHM: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;     // To light walls or not
const TORCH_RADIUS: i32 = 10;

const LIMIT_FPS: i32 = 20;      // FPS maximum


type Map = Vec<Vec<tile::Tile>>;

#[derive(serde::Serialize, serde::Deserialize)]
struct Game {
    map: Map,
    messages: Messages,
    inventory: Vec<Object>,
    dungeon_level: u32,
}

struct Tcod {
    root: Root,
    con: Offscreen,
    panel: Offscreen,
    fov: FovMap,
    key: Key,
    mouse: Mouse,
}


#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}


fn mut_two<T>(first_index: usize, second_index: usize, items: &mut [T]) -> (&mut T, &mut T) {
    assert!(first_index != second_index);
    let split_at_index = cmp::max(first_index, second_index);
    let (first_slice, second_slice) = items.split_at_mut(split_at_index);

    if first_index < second_index {
        (&mut first_slice[first_index], &mut second_slice[0])
    } else {
        (&mut second_slice[0], &mut first_slice[second_index])
    }
}

fn handle_keys(tcod: &mut Tcod, game: &mut Game, objects: &mut Vec<Object>) -> PlayerAction {
    use tcod::input::KeyCode::*;
    use PlayerAction::*;

    let player_alive = objects[PLAYER].alive;
    match (tcod.key, tcod.key.text(), player_alive) {
        (Key { code: Enter, alt: true, .. }, _, _) => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
            return DidntTakeTurn
        }
        (Key { code: Escape, .. }, _, _) => return Exit,    // Exit game

        // Movement keys
        (Key { code: Up, .. }, _, true) | (Key { code: NumPad8, .. }, _, true) => {
            player_move_or_attack(0, -1, game, objects);
            TookTurn
        }
        (Key { code: Down, .. }, _, true) | (Key { code: NumPad2, .. }, _, true) => {
            player_move_or_attack(0, 1, game, objects);
            TookTurn
        }
        (Key { code: Left, .. }, _, true) | (Key { code: NumPad4, .. }, _, true) => {
            player_move_or_attack(-1, 0, game, objects);
            TookTurn
        }
        (Key { code: Right, .. }, _, true) | (Key { code: NumPad6, .. }, _, true) => {
            player_move_or_attack(1, 0, game, objects);
            TookTurn
        }
        (Key { code: Home, .. }, _, true) | (Key { code: NumPad7, .. }, _, true) => {
            player_move_or_attack(-1, -1, game, objects);
            TookTurn
        }
        (Key { code: PageUp, .. }, _, true) | (Key { code: NumPad9, .. }, _, true) => {
            player_move_or_attack(1, -1, game, objects);
            TookTurn
        }
        (Key { code: End, .. }, _, true) | (Key { code: NumPad1, .. }, _, true) => {
            player_move_or_attack(-1, 1, game, objects);
            TookTurn
        }
        (Key { code: PageDown, .. }, _, true) | (Key { code: NumPad3, .. }, _, true) => {
            player_move_or_attack(1, 1, game, objects);
            TookTurn
        }
        (Key { code: NumPad5, .. }, _, true) => {
            TookTurn // Do nothing, i.e. wait for the monster to come to you
        }

        // Pick up an item
        (Key { code: Text, ..}, "e", true) => {
            let item_id = objects
                .iter()
                .position(|object| object.pos() == objects[PLAYER].pos() && object.item.is_some());

            if let Some(item_id) = item_id {
                pick_item_up(item_id, game, objects);
            }
            DidntTakeTurn
        }

        // Access inventory
        (Key { code: Text, ..}, "i", true) => {
            // Show inventory
            let inventory_index = inventory_menu(&game.inventory, "Press the key next to an item to use it, or any other to cancel", &mut tcod.root);

            if let Some(inventory_index) = inventory_index {
                use_item(inventory_index, tcod, game, objects);
            }
            DidntTakeTurn
        }

        // Drop item from inventory
        (Key { code: Text, ..}, "d", true) => {
            // Show the inventory, drop item if selected
            let inventory_index = inventory_menu(&game.inventory, "Press the key next to an item to drop it, or any other to cancel.\n", &mut tcod.root);
            if let Some(inventory_index) = inventory_index {
                drop_item(inventory_index, game, objects);
            }
            DidntTakeTurn
        }

        // Access Stairs
        (Key { code: Enter, ..}, _, true) => {
            // Go down stairs
            let player_on_stairs = objects
                .iter()
                .any(|obj| obj.pos() == objects[PLAYER].pos() && obj.name == "Stairs");
            if player_on_stairs {
                next_level(tcod, game, objects);
            }
            DidntTakeTurn
        }

        // Show stats
        (Key { code: Text, .. }, "c", true) => {
            // show character information
            let player = &objects[PLAYER];
            let level = player.level;
            let level_up_xp = LEVEL_UP_BASE + player.level as i32 * LEVEL_UP_FACTOR;
            if let Some(fighter) = player.fighter.as_ref() {
                let msg = format!(
                    "Character information
        
        Level: {}
        Experience: {}
        Experience to level up: {}
        
        Maximum HP: {}
        Attack: {}
        Defense: {}",
                    level, fighter.xp, level_up_xp, fighter.max_hp, fighter.power, fighter.defense
                );
                msgbox(&msg, CHARACTER_SCREEN_WIDTH, &mut tcod.root);
            }
        
            DidntTakeTurn
        }

        _ => return DidntTakeTurn,
    }
}

fn get_names_under_mouse(mouse: Mouse, objects: &[Object], fov_map: &FovMap) -> String {
    let (x, y) = (mouse.cx as i32, mouse.cy as i32);

    // Create a list with the names of all objects at mouse's coords and in FOV
    let names = objects
        .iter()
        .filter(|obj| obj.pos() == (x,y) && fov_map.is_in_fov(obj.x, obj.y))
        .map(|obj| obj.name.clone())
        .collect::<Vec<_>>();

    names.join(", ")
}

fn main() {
    tcod::system::set_fps(LIMIT_FPS);

    let root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Roguelike")
        .init();

    let mut tcod = Tcod { 
        root, 
        con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT), 
        panel: Offscreen::new(SCREEN_WIDTH, PANEL_HEIGHT),
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
        key: Default::default(),
        mouse: Default::default(), 
    };

   main_menu(&mut tcod);
}