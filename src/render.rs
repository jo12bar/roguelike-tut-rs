use rltk::{Rltk, RGB};
use specs::prelude::*;

use crate::{Map, Position, Renderable, TileType, DEBUG_MAP_VIEW};

/// Draw a game map on screen. Only draws tiles visible within the player's viewshed.
pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();

    let mut y = 0;
    let mut x = 0;

    for (idx, tile) in map.tiles.iter().enumerate() {
        // Render a tile depending on the tile type
        if map.revealed_tiles[idx] || DEBUG_MAP_VIEW {
            let glyph;
            let mut fg;

            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    fg = RGB::from_f32(0.0, 0.5, 0.5);
                }
                TileType::Wall => {
                    glyph = wall_glyph(&map, x, y);
                    fg = RGB::from_f32(0.0, 1.0, 0.0);
                }
                TileType::DownStairs => {
                    glyph = rltk::to_cp437('>');
                    fg = RGB::from_f32(0.0, 1.0, 1.0);
                }
            }

            // If the tile isn't _currently_ visible to the player, grey it out
            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale();
            }

            ctx.set(x, y, fg, RGB::from_f32(0.0, 0.0, 0.0), glyph);
        }

        // Next coord
        x += 1;
        if x > map.width - 1 {
            x = 0;
            y += 1;
        }
    }
}

/// Render any entity that has [`Position`] and [`Renderable`].
pub fn draw_entities(ecs: &World, ctx: &mut Rltk) {
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let map = ecs.fetch::<Map>();

    let mut data = (&positions, &renderables).join().collect::<Vec<_>>();

    // Sort entities by render order, so we render lower entities underneath higher entities.
    data.sort_unstable_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));

    for (pos, render) in data {
        // Only render the entity if the player can currently see it!
        let idx = map.xy_idx(pos.x, pos.y);
        if map.visible_tiles[idx] || DEBUG_MAP_VIEW {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

fn wall_glyph(map: &Map, x: i32, y: i32) -> rltk::FontCharType {
    if x < 1 || x > map.width - 1 || y < 1 || y > map.height - 1_i32 {
        return 35;
    }
    const NORTH: u8 = 0b0000_0001;
    const SOUTH: u8 = 0b0000_0010;
    const WEST: u8 = 0b0000_0100;
    const EAST: u8 = 0b0000_1000;
    let mut mask: u8 = 0b0000_0000;

    if is_revealed_and_wall(map, x, y - 1) {
        mask |= NORTH;
    }
    if is_revealed_and_wall(map, x, y + 1) {
        mask |= SOUTH;
    }
    if is_revealed_and_wall(map, x - 1, y) {
        mask |= WEST;
    }
    if is_revealed_and_wall(map, x + 1, y) {
        mask |= EAST;
    }

    macro_rules! switch {
        ($v:expr; $($a:expr => $b:expr,)* _ => $e:expr $(,)?) => {
            match $v {
                $(v if v == $a => $b,)*
                _ => $e,
            }
        };
    }

    switch! {mask;
        0 => 9,    // Pillar because we can't see neighbors
        NORTH => 186,  // Wall only to the north
        SOUTH => 186,  // Wall only to the south
        NORTH | SOUTH => 186,  // Wall to the north and south
        WEST => 205,  // Wall only to the west
        WEST | NORTH => 188,  // Wall to the north and west
        WEST | SOUTH => 187,  // Wall to the south and west
        WEST | NORTH | SOUTH => 185,  // Wall to the north, south and west
        EAST => 205,  // Wall only to the east
        EAST | NORTH => 200,  // Wall to the north and east
        EAST | SOUTH => 201, // Wall to the south and east
        EAST | NORTH | SOUTH => 204, // Wall to the north, south and east
        EAST | WEST => 205, // Wall to the east and west
        EAST | WEST | SOUTH => 203, // Wall to the east, west, and south
        EAST | WEST | NORTH => 202, // Wall to the east, west, and north
        NORTH | EAST | SOUTH | WEST => 206, // ╬ Wall on all sides
        _ => 35,   // We missed one?
    }
}

fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
    let idx = map.xy_idx(x, y);
    map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
}
