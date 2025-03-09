use tcod::colors::*;
use crate::*;
use rand::Rng;
use std::cmp;

pub fn make_map(objects: &mut Vec<Object>, level: u32) -> Map {
    let mut map = vec![vec![tile::Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    let mut rooms = vec![];

    // Ensures no objects are lodged in walls when making new level
    // ! Works only when the player is the first object
    assert_eq!(&objects[PLAYER] as *const _, &objects[0] as *const _);
    objects.truncate(1);

    let mut room_count = 0;
    while room_count < MIN_ROOMS && room_count < MAX_ROOMS {
        // Generate random room dimensions
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);

        // Generate random room position
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

        let new_room = tile::Rect::new(x, y, w, h);
        let failed = rooms.iter().any(|other_room| new_room.intersects_with(other_room));

        if !failed {
            create_room(new_room, &mut map);
            place_objects(new_room, &map, objects, level);
            let (new_x, new_y) = new_room.center(); 

            if rooms.is_empty() {
                // First room, place the player here
                objects[PLAYER].set_pos(new_x, new_y);
            } else {
                // Connect new room to previous room
                let (prev_x, prev_y) = rooms.last().unwrap().center();
                connect_rooms(prev_x, prev_y, new_x, new_y, &mut map);
            }

            rooms.push(new_room);
            room_count += 1;
        }
    }

    // Ensure at least one room exists before placing stairs
    if let Some(last_room) = rooms.last() {
        let (last_room_x, last_room_y) = last_room.center();
        let mut stairs = Object::new(last_room_x, last_room_y, '<', WHITE.into(), "Stairs", false);
        stairs.always_visible = true;
        objects.push(stairs);
    } else {
        panic!("No rooms were placed! Unable to generate stairs.");
    }

    // Verify all rooms are connected
    ensure_map_connectivity(&mut map, &rooms);

    map
}

pub fn create_room(room: tile::Rect, map: &mut Map) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = tile::Tile::empty();
        }
    }
}

pub fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    for x in cmp::min(x1, x2)..=cmp::max(x1, x2) {
        map[x as usize][y as usize] = tile::Tile::empty();
    }
}

pub fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    for y in cmp::min(y1, y2)..=cmp::max(y1, y2) {
        map[x as usize][y as usize] = tile::Tile::empty();
    }
}

pub fn connect_rooms(x1: i32, y1: i32, x2: i32, y2: i32, map: &mut Map) {
    if rand::random() {
        create_h_tunnel(x1, x2, y1, map);
        create_v_tunnel(y1, y2, x2, map);
    } else {
        create_v_tunnel(y1, y2, x1, map);
        create_h_tunnel(x1, x2, y2, map);
    }
}

pub fn ensure_map_connectivity(map: &mut Map, rooms: &Vec<tile::Rect>) {
    let mut visited = vec![vec![false; MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    // Flood fill from the player's starting room
    let (start_x, start_y) = rooms[0].center();
    flood_fill(start_x, start_y, map, &mut visited);

    for room in rooms {
        let (room_x, room_y) = room.center();
        if !visited[room_x as usize][room_y as usize] {
            // If room is not connected, force-connect it
            let (closest_x, closest_y) = find_closest_connected_tile(room_x, room_y, &visited);
            connect_rooms(closest_x, closest_y, room_x, room_y, map);
        }
    }
}

fn flood_fill(x: i32, y: i32, map: &Map, visited: &mut Vec<Vec<bool>>) {
    let mut stack = vec![(x, y)];
    while let Some((cx, cy)) = stack.pop() {
        if cx < 0 || cy < 0 || cx >= MAP_WIDTH || cy >= MAP_HEIGHT {
            continue;
        }
        if visited[cx as usize][cy as usize] || !map[cx as usize][cy as usize].is_walkable() {
            continue;
        }

        visited[cx as usize][cy as usize] = true;
        stack.push((cx + 1, cy));
        stack.push((cx - 1, cy));
        stack.push((cx, cy + 1));
        stack.push((cx, cy - 1));
    }
}

fn find_closest_connected_tile(x: i32, y: i32, visited: &Vec<Vec<bool>>) -> (i32, i32) {
    let mut queue = vec![(x, y)];
    while let Some((cx, cy)) = queue.pop() {
        if visited[cx as usize][cy as usize] {
            return (cx, cy);
        }
        if cx > 0 {
            queue.push((cx - 1, cy));
        }
        if cy > 0 {
            queue.push((cx, cy - 1));
        }
        if cx < MAP_WIDTH - 1 {
            queue.push((cx + 1, cy));
        }
        if cy < MAP_HEIGHT - 1 {
            queue.push((cx, cy + 1));
        }
    }
    (x, y)
}
