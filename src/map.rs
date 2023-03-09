use std::cmp::{max, min};

use rltk::{RandomNumberGenerator, Rltk, RGB};

use crate::Rect;

/// All possible tile types.
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

/// A level map. This includes all the tiles, rooms, and so on that constitute
/// the level's layout.
pub struct Map {
    /// An array of all map tiles.
    ///
    /// Use [`Self::xy_idx()`] to convert (x, y) coordinates into indexes in this array.
    pub tiles: Vec<TileType>,

    /// A list of all rooms contained in this map.
    pub rooms: Vec<Rect>,

    /// The map's width.
    pub width: i32,
    /// The map's height.
    pub height: i32,
}

impl Map {
    /// Convert (x, y) coordinates to an index into [`Self::tiles`].
    pub const fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    /// Add a rectangular room made entirely of [`TileType::Floor`].
    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    /// Make a horizontal tunnel between two x-coordinates at a specific y-coordinate.
    /// The tunnel is made entirely of [`TileType::Floor`].
    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < 80 * 50 {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    /// Make a vertical tunnel between two y-coordinates at a specific x-coordinate.
    /// The tunnel is made entirely of [`TileType::Floor`].
    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < 80 * 50 {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    /// Create a new map with randomly-placed rooms that are connected by corridors.
    ///
    /// The map will have a width of 80 and a height of 50.
    /// This uses the algorithm from http://rogueliketutorials.com/tutorials/tcod/part-3/.
    pub fn new_map_rooms_and_corridors() -> Self {
        let mut map = Self {
            tiles: vec![TileType::Wall; 80 * 50],
            rooms: Vec::new(),
            width: 80,
            height: 50,
        };

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, 80 - w - 1) - 1;
            let y = rng.roll_dice(1, 50 - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);

            if !map
                .rooms
                .iter()
                .any(|other_room| new_room.intersect(other_room))
            {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map
    }
}

/// Draw a game map on screen.
pub fn draw_map(map: &Map, ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.tiles.iter() {
        // Render a tile depending on the tile type
        match tile {
            TileType::Floor => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.5, 0.5, 0.5),
                    RGB::from_f32(0.0, 0.0, 0.0),
                    rltk::to_cp437('.'),
                );
            }
            TileType::Wall => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.0, 1.0, 0.0),
                    RGB::from_f32(0.0, 0.0, 0.0),
                    rltk::to_cp437('#'),
                );
            }
        }

        // Next coord
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}
