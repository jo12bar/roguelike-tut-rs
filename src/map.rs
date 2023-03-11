use std::cmp::{max, min};

use rltk::{Algorithm2D, BaseMap, Point, RandomNumberGenerator};
use specs::Entity;

use crate::Rect;

// Note: we don't make these constants public so that other modules are forced
// to use references to a `Map`.
const MAPWIDTH: usize = 80;
const MAPHEIGHT: usize = 43;
const MAPSIZE: usize = MAPWIDTH * MAPHEIGHT;

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

    /// All tiles that the player has revealed during their explorations.
    ///
    /// An element in this vector will be `true` if the player has revealed the
    /// corresponding tile in [`Self::tiles`].
    pub revealed_tiles: Vec<bool>,

    /// All tiles that are _currently_ visible to the player.
    ///
    /// This allows greying out tiles that were previously revealed but are no
    /// longer visible.
    ///
    /// An element in this vector will be `true` if the player can currently see the
    /// corresponding tile in [`Self::tiles`].
    pub visible_tiles: Vec<bool>,

    /// All tiles that are blocked from access. This includes things like walls,
    /// monsters, etc. that can't be moved onto by other entities.
    ///
    /// An element in this vector will be `true` if the corresponding tile in
    /// [`Self::tiles`] is blocked from access.
    pub blocked: Vec<bool>,

    /// A record of which entities are present in each tile of the map.
    pub tile_content: Vec<Vec<Entity>>,
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
            if idx > 0 && idx < MAPSIZE {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    /// Make a vertical tunnel between two y-coordinates at a specific x-coordinate.
    /// The tunnel is made entirely of [`TileType::Floor`].
    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < MAPSIZE {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    /// Populate [`Self::blocked`] with all statically-blocked tiles.
    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    /// Clear out all entity handles in every tile location from [`Self::tile_content`].
    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    /// Create a new map with randomly-placed rooms that are connected by corridors.
    ///
    /// The map will have a width of 80 and a height of 50.
    /// This uses the algorithm from http://rogueliketutorials.com/tutorials/tcod/part-3/.
    pub fn new_map_rooms_and_corridors(rng: &mut RandomNumberGenerator) -> Self {
        let mut map = Self {
            tiles: vec![TileType::Wall; MAPSIZE],
            rooms: Vec::new(),
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles: vec![false; MAPSIZE],
            visible_tiles: vec![false; MAPSIZE],
            blocked: vec![false; MAPSIZE],
            tile_content: vec![Vec::new(); MAPSIZE],
        };

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, MAPWIDTH as i32 - w - 1) - 1;
            let y = rng.roll_dice(1, MAPHEIGHT as i32 - h - 1) - 1;
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

    /// Returns `true` if a particular tile can be entered ("walked on") by an entity.
    ///
    /// Named "is_exit_valid" as it's used to implement [`BaseMap::get_available_exits`], which
    /// is used for pathfinding.
    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        // bounds check
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }

        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();

        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Check cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0))
        };

        // Check diagonals
        if self.is_exit_valid(x - 1, y - 1) {
            exits.push(((idx - w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push(((idx - w) + 1, 1.45));
        }
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push(((idx + w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push(((idx + w) + 1, 1.45));
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);

        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}
