//use crate::{spire::*, LEVEL_SIZE, TILE_SIZE};
//use bevy::prelude::*;
//use bevy::utils::hashbrown::HashMap;
//use bevy_ldtk_scene::{levels::LevelSet, prelude::LevelMetaExt, world::LevelUid};
//use rand::Rng;
//
//#[derive(Default, Debug, Clone)]
//pub struct MapGen<const C: usize, const R: usize> {
//    map: Map<C, R>,
//}
//
//impl<const C: usize, const R: usize> MapGen<C, R> {
//    pub fn gen(self) -> Map<C, R> {
//        let key = RoomKey::all();
//
//        let mut map = self.map.clone();
//        while map
//            .rooms
//            .last()
//            .unwrap()
//            .iter()
//            .all(|r| r.is_none_or(|r| r.faces.up().closed()))
//        {
//            map = self.map.clone();
//            let starting_room =
//                RoomFaces::new(Face::Open, Face::Closed, Face::Closed, Face::Closed);
//            map.rooms[0][1] = Some(starting_room.into());
//            Self::gen_recurse(&mut map, &key, &starting_room, Direction::Up, 1, 0);
//        }
//
//        map
//    }
//
//    fn gen_recurse(
//        map: &mut Map<C, R>,
//        key: &RoomKey,
//        current_room: &RoomFaces,
//        prev_dir: Direction,
//        current_x: usize,
//        current_y: usize,
//    ) {
//        let valid_rooms = key.joining_rooms(current_room).unwrap();
//        for dir in [
//            Direction::Up,
//            Direction::Down,
//            Direction::Right,
//            Direction::Left,
//        ]
//        .iter()
//        {
//            if *dir == prev_dir.opposite() {
//                continue;
//            }
//
//            let x = dir.x() + current_x as i32;
//            if x < 0 || x >= C as i32 {
//                continue;
//            }
//
//            let y = dir.y() + current_y as i32;
//            if y < 0 || y >= R as i32 {
//                continue;
//            }
//
//            let Some(rooms) = valid_rooms.get(dir) else {
//                continue;
//            };
//
//            if rooms.is_empty() || map.rooms[y as usize][x as usize].is_some() {
//                continue;
//            }
//
//            let new_room = rooms[rand::thread_rng().gen_range(0..rooms.len())];
//            map.rooms[y as usize][x as usize] = Some(new_room.into());
//            Self::gen_recurse(map, key, &new_room, *dir, x as usize, y as usize);
//
//            // let mut depth = 0;
//            // loop {
//            //     depth += 1;
//            //
//            //     if depth > max_depth {
//            //         break;
//            //     }
//            //
//            //     if current_x as i32 + dir.x() > C as i32
//            //         || current_y as i32 + dir.y() > R as i32
//            //         || current_x as i32 + dir.x() < 0
//            //         || current_y as i32 + dir.y() < 0
//            //     {
//            //         break;
//            //     }
//            //
//            //     if *dir == prev_dir.opposite() {
//            //         continue;
//            //     }
//            //
//            //     if let Some(rooms) = valid_rooms.get(dir) {
//            //         if rooms.is_empty() {
//            //             continue;
//            //         }
//            //
//            //         let x = {
//            //             let x = dir.x();
//            //             if current_x as i32 + x < 0 {
//            //                 continue;
//            //             } else {
//            //                 (current_x as i32 + x) as usize
//            //             }
//            //         };
//            //         let y = {
//            //             let y = dir.y();
//            //             if current_y as i32 + y < 0 {
//            //                 continue;
//            //             } else {
//            //                 (current_y as i32 + y) as usize
//            //             }
//            //         };
//            //
//            //         if let Some(room) = map.rooms.get_mut(y).and_then(|r| r.get_mut(x)) {
//            //             let new_room = rooms[rand::thread_rng().gen_range(0..rooms.len())];
//            //             *room = new_room.into();
//            //             Self::gen_recurse(map, key, &new_room, *dir, x, y);
//            //             break;
//            //         }
//            //     }
//            // }
//        }
//    }
//}
//
//#[derive(Debug, Clone)]
//pub struct Map<const C: usize, const R: usize> {
//    rooms: [[Option<Room>; C]; R],
//}
//
//impl<const C: usize, const R: usize> Default for Map<C, R> {
//    fn default() -> Self {
//        Self {
//            rooms: [[None; C]; R],
//        }
//    }
//}
//
//impl<const C: usize, const R: usize> LevelSet for Map<C, R> {
//    fn into_loader(
//        self,
//        registry: &mut bevy_ldtk_scene::levels::LevelMetaRegistry,
//    ) -> Vec<(LevelUid, bevy::prelude::Vec3)> {
//        let mut output = Vec::with_capacity(R * C);
//
//        registry.register(L1::uid(), L1::meta());
//        registry.register(L2::uid(), L2::meta());
//        registry.register(L3::uid(), L3::meta());
//        registry.register(L4::uid(), L4::meta());
//        registry.register(L5::uid(), L5::meta());
//        registry.register(L6::uid(), L6::meta());
//        registry.register(L7::uid(), L7::meta());
//        registry.register(L8::uid(), L8::meta());
//        registry.register(L9::uid(), L9::meta());
//        registry.register(L10::uid(), L10::meta());
//        registry.register(L11::uid(), L11::meta());
//        registry.register(L12::uid(), L12::meta());
//        registry.register(L13::uid(), L13::meta());
//        registry.register(L14::uid(), L14::meta());
//        registry.register(L15::uid(), L15::meta());
//        registry.register(L16::uid(), L16::meta());
//
//        for y in 0..R {
//            for x in 0..C {
//                output.push((
//                    self.rooms[y][x]
//                        .map(|room| room.faces.level())
//                        .unwrap_or(L16::uid()),
//                    (TILE_SIZE * LEVEL_SIZE * Vec2::new(x as f32, -(y as f32))).extend(0.),
//                ));
//            }
//        }
//
//        output
//    }
//}
//
//impl<const C: usize, const R: usize> Map<C, R> {
//    pub fn trim_edge(mut self) -> Self {
//        for y in 0..R {
//            for x in 0..C {
//                if x == 0 {
//                    if let Some(room) = &mut self.rooms[y][x] {
//                        *room.faces.left_mut() = Face::Closed;
//                    }
//                } else if x == C - 1 {
//                    if let Some(room) = &mut self.rooms[y][x] {
//                        *room.faces.right_mut() = Face::Closed;
//                    }
//                }
//            }
//        }
//
//        self
//    }
//}
//
//#[derive(Default, Debug, Clone, Copy)]
//pub struct Room {
//    faces: RoomFaces,
//}
//
//impl From<RoomFaces> for Room {
//    fn from(value: RoomFaces) -> Self {
//        Self { faces: value }
//    }
//}
//
//impl Room {
//    pub fn new(faces: RoomFaces) -> Self {
//        Self { faces }
//    }
//}
//
//#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
//pub struct RoomFaces([Face; 4]);
//
//impl RoomFaces {
//    pub fn splat(face: Face) -> Self {
//        Self([face; 4])
//    }
//
//    pub fn new(up: Face, down: Face, right: Face, left: Face) -> Self {
//        Self([left, right, up, down])
//    }
//
//    /// Is `other` conntected with `self` with respect to `dir`.
//    pub fn connected(&self, other: &Self, dir: Direction) -> bool {
//        match dir {
//            Direction::Up => self.up().open() && other.down().open(),
//            Direction::Down => self.down().open() && other.up().open(),
//            Direction::Right => self.right().open() && other.left().open(),
//            Direction::Left => self.left().open() && other.right().open(),
//        }
//    }
//
//    pub fn left(&self) -> Face {
//        self.0[0]
//    }
//
//    pub fn right(&self) -> Face {
//        self.0[1]
//    }
//
//    pub fn up(&self) -> Face {
//        self.0[2]
//    }
//
//    pub fn down(&self) -> Face {
//        self.0[3]
//    }
//
//    pub fn left_mut(&mut self) -> &mut Face {
//        &mut self.0[0]
//    }
//
//    pub fn right_mut(&mut self) -> &mut Face {
//        &mut self.0[1]
//    }
//
//    pub fn up_mut(&mut self) -> &mut Face {
//        &mut self.0[2]
//    }
//
//    pub fn down_mut(&mut self) -> &mut Face {
//        &mut self.0[3]
//    }
//
//    pub fn level(&self) -> LevelUid {
//        match (self.up(), self.right(), self.down(), self.left()) {
//            (Face::Closed, Face::Open, Face::Open, Face::Closed) => L1::uid(),
//            (Face::Closed, Face::Open, Face::Open, Face::Open) => L2::uid(),
//            (Face::Closed, Face::Closed, Face::Open, Face::Open) => L3::uid(),
//            (Face::Open, Face::Open, Face::Open, Face::Closed) => L4::uid(),
//            (Face::Open, Face::Open, Face::Open, Face::Open) => L5::uid(),
//            (Face::Open, Face::Closed, Face::Open, Face::Open) => L6::uid(),
//            (Face::Open, Face::Open, Face::Closed, Face::Closed) => L7::uid(),
//            (Face::Open, Face::Open, Face::Closed, Face::Open) => L8::uid(),
//            (Face::Open, Face::Closed, Face::Closed, Face::Open) => L9::uid(),
//            (Face::Closed, Face::Open, Face::Closed, Face::Closed) => L10::uid(),
//            (Face::Closed, Face::Open, Face::Closed, Face::Open) => L11::uid(),
//            (Face::Closed, Face::Closed, Face::Closed, Face::Open) => L12::uid(),
//            (Face::Closed, Face::Closed, Face::Open, Face::Closed) => L13::uid(),
//            (Face::Open, Face::Closed, Face::Open, Face::Closed) => L14::uid(),
//            (Face::Open, Face::Closed, Face::Closed, Face::Closed) => L15::uid(),
//            (Face::Closed, Face::Closed, Face::Closed, Face::Closed) => L16::uid(),
//        }
//    }
//}
//
//#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
//pub enum Face {
//    Open,
//    #[default]
//    Closed,
//}
//
//impl Face {
//    pub fn from_usize(face: usize) -> Option<Self> {
//        match face {
//            0 => Some(Self::Open),
//            1 => Some(Self::Closed),
//            _ => None,
//        }
//    }
//
//    pub fn open(&self) -> bool {
//        matches!(self, Self::Open)
//    }
//
//    pub fn closed(&self) -> bool {
//        matches!(self, Self::Closed)
//    }
//}
//
//#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//pub enum Direction {
//    Up,
//    Down,
//    Left,
//    Right,
//}
//
//impl Direction {
//    pub fn x(&self) -> i32 {
//        match self {
//            Self::Right => 1,
//            Self::Left => -1,
//            _ => 0,
//        }
//    }
//
//    pub fn y(&self) -> i32 {
//        match self {
//            Self::Up => 1,
//            Self::Down => -1,
//            _ => 0,
//        }
//    }
//
//    pub fn opposite(&self) -> Self {
//        match self {
//            Self::Up => Self::Down,
//            Self::Down => Self::Up,
//            Self::Right => Self::Left,
//            Self::Left => Self::Right,
//        }
//    }
//}
//
//#[derive(Debug)]
//pub struct RoomKey(HashMap<RoomFaces, HashMap<Direction, Vec<RoomFaces>>>);
//
//impl RoomKey {
//    fn all() -> Self {
//        let mut key = HashMap::default();
//
//        let mut rooms = Vec::new();
//        for up in 0..2 {
//            for down in 0..2 {
//                for left in 0..2 {
//                    for right in 0..2 {
//                        rooms.push(RoomFaces::new(
//                            Face::from_usize(up).unwrap(),
//                            Face::from_usize(down).unwrap(),
//                            Face::from_usize(right).unwrap(),
//                            Face::from_usize(left).unwrap(),
//                        ));
//                    }
//                }
//            }
//        }
//
//        for room in rooms.iter() {
//            let mut compatible = HashMap::default();
//
//            compatible.insert(Direction::Up, {
//                room.up()
//                    .open()
//                    .then_some(rooms.iter().filter(|r| r.down().open()).copied().collect())
//                    .unwrap_or_default()
//            });
//            compatible.insert(Direction::Down, {
//                room.down()
//                    .open()
//                    .then_some(rooms.iter().filter(|r| r.up().open()).copied().collect())
//                    .unwrap_or_default()
//            });
//            compatible.insert(Direction::Left, {
//                room.left()
//                    .open()
//                    .then_some(rooms.iter().filter(|r| r.right().open()).copied().collect())
//                    .unwrap_or_default()
//            });
//            compatible.insert(Direction::Right, {
//                room.right()
//                    .open()
//                    .then_some(rooms.iter().filter(|r| r.left().open()).copied().collect())
//                    .unwrap_or_default()
//            });
//
//            key.insert(*room, compatible);
//        }
//
//        Self(key)
//    }
//
//    pub fn joining_rooms(&self, faces: &RoomFaces) -> Option<&HashMap<Direction, Vec<RoomFaces>>> {
//        self.0.get(faces)
//    }
//}
//
//pub fn draw<const C: usize, const R: usize>(map: &Map<C, R>) {
//    println!("#######################################");
//    println!("         The W A I L I N G Tower       ");
//    println!("#######################################");
//
//    for row in map.rooms.iter().rev() {
//        println!();
//        for room in row.iter() {
//            if let Some(room) = room {
//                print!("█{}█", draw_code(room.faces.up()));
//            } else {
//                print!("███");
//            }
//        }
//        println!();
//        for room in row.iter() {
//            if let Some(room) = room {
//                print!(
//                    "{} {}",
//                    draw_code(room.faces.left()),
//                    draw_code(room.faces.right())
//                );
//            } else {
//                print!("███");
//            }
//        }
//        println!();
//        for room in row.iter() {
//            if let Some(room) = room {
//                print!("█{}█", draw_code(room.faces.down()));
//            } else {
//                print!("███");
//            }
//        }
//    }
//
//    println!();
//}
//
//fn draw_code(face: Face) -> &'static str {
//    match face {
//        Face::Open => " ",
//        Face::Closed => "█",
//    }
//}
//
//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    #[test]
//    fn key_all() {
//        let key = RoomKey::all();
//        for (room, valid_paths) in key.0.iter() {
//            for (dir, rooms) in valid_paths.iter() {
//                for other in rooms.iter() {
//                    assert!(room.connected(other, *dir));
//                }
//            }
//        }
//
//        let map = MapGen::<3, 5>::default().gen();
//        draw(&map);
//        draw(&map.trim_edge());
//        // draw(&MapGen::<3, 5>::default().gen());
//        // draw(&MapGen::<3, 5>::default().gen());
//    }
//}
