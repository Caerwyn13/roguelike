use rand::{distributions::{IndependentSample, Weighted, WeightedChoice}, Rng};
use tcod::{colors::*, console::*};

use crate::*;


#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Object {
    pub x: i32,      
    pub y: i32,         
    pub char: char,     
    pub colour: SerializableColour,  
    pub name: String,
    pub blocks: bool,
    pub alive: bool,
    pub fighter: Option<Fighter>,
    pub ai: Option<Ai>,
    pub item: Option<Item>,
    pub always_visible: bool,
    pub level: u32,
    pub equipment: Option<Equipment>,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, colour: SerializableColour, name: &str, blocks: bool) -> Self {
        Object { 
            x, 
            y, 
            char, 
            colour, 
            name: name.into(), 
            blocks, 
            alive: false, 
            fighter: None, 
            ai: None, 
            item: None, 
            always_visible: false, 
            level: 1,
            equipment: None, 
        }
    }

    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.colour.into());
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }

    pub fn distance(&self, x: i32, y: i32) -> f32 {
        (((x - self.x).pow(2) + (y - self.y).pow(2)) as f32).sqrt()
    }

    pub fn distance_to(&self, other: &Object) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
    }
}


#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Fighter {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
    pub xp: i32,
    pub on_death: DeathCallback,
}


pub fn place_objects(room: Rect, map: &Map, objects: &mut Vec<Object>, level: u32) {
    let max_monsters = from_dungeon_level(&[
        Transition { level: 1, value: 2 },
        Transition { level: 4, value: 3 },
        Transition { level: 6, value: 5 },
    ], level);
    
    let num_monsters = rand::thread_rng().gen_range(0, max_monsters);

    let troll_chance = from_dungeon_level(&[
        Transition { level: 3, value: 15 },
        Transition { level: 5, value: 30 },
        Transition { level: 7, value: 60 },
    ], level);

    let mut monster_choices = vec![
        Weighted { weight: 80, item: "orc" },
        Weighted { weight: troll_chance, item: "troll" },
    ];
    let monster_choice = WeightedChoice::new(&mut monster_choices);
    
    for _ in 0..num_monsters {
        if let Some((x, y)) = find_unblocked_position(&room, map, objects) {
            let monster = create_monster(x, y, monster_choice.ind_sample(&mut rand::thread_rng()));
            objects.push(monster);
        }
    }

    let max_items = from_dungeon_level(&[
        Transition { level: 1, value: 1 },
        Transition { level: 4, value: 2 },
    ], level);
    
    let mut item_choices = vec![
        Weighted { weight: 35, item: Item::Heal },
        Weighted { weight: from_dungeon_level(&[Transition { level: 4, value: 25 }], level), item: Item::Lightning },
        Weighted { weight: from_dungeon_level(&[Transition { level: 6, value: 25 }], level), item: Item::Fireball },
        Weighted { weight: from_dungeon_level(&[Transition { level: 2, value: 10 }], level), item: Item::Confuse },
        Weighted { weight: 1000, item: Item::Equipment }, // TODO: Adjust weight
    ];
    let item_choice = WeightedChoice::new(&mut item_choices);
    let num_items = rand::thread_rng().gen_range(0, max_items);

    for _ in 0..num_items {
        if let Some((x, y)) = find_unblocked_position(&room, map, objects) {
            let item = create_item(x, y, item_choice.ind_sample(&mut rand::thread_rng()));
            objects.push(item);
        }
    }
}

fn find_unblocked_position(room: &Rect, map: &Map, objects: &Vec<Object>) -> Option<(i32, i32)> {
    let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
    let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);
    if !is_blocked(x, y, map, objects) {
        Some((x, y))
    } else {
        None
    }
}

fn create_monster(x: i32, y: i32, monster_type: &str) -> Object {
    let (char, colour, name, fighter) = match monster_type {
        "orc" => ('o', DESATURATED_GREEN.into(), "orc", Fighter { max_hp: 20, hp: 20, defense: 0, power: 4, xp: 35, on_death: DeathCallback::Monster }),
        "troll" => ('T', DARKER_GREEN.into(), "troll", Fighter { max_hp: 30, hp: 30, defense: 2, power: 8, xp: 100, on_death: DeathCallback::Monster }),
        _ => unreachable!(),
    };
    let mut monster = Object::new(x, y, char, colour, name, true);
    monster.fighter = Some(fighter);
    monster.ai = Some(Ai::Basic);
    monster.alive = true;
    monster
}

fn create_item(x: i32, y: i32, item_type: Item) -> Object {
    let (char, colour, name) = match item_type {
        Item::Equipment => ('/', SKY.into(), "Sword"),
        Item::Heal => ('!', VIOLET.into(), "Healing Potion"),
        Item::Lightning => ('#', LIGHT_YELLOW.into(), "Scroll of Lightning bolt"),
        Item::Fireball => ('#', DARKER_ORANGE.into(), "Scroll of Fireball"),
        Item::Confuse => ('#', PURPLE.into(), "Scroll of Confusion"),
    };
    let mut object = Object::new(x, y, char, colour, name, false);
    object.item = Some(item_type);
    if let Item::Equipment = item_type {
        object.equipment = Some(Equipment { equipped: false, slot: Slot::RightHand });
    }
    object
}