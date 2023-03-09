use rltk::{Rltk, RGB};
use specs::World;

use crate::{Map, TileType};

/// Draw a game map on screen. Only draws tiles visible within the player's viewshed.
pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();

    let mut y = 0;
    let mut x = 0;

    for (idx, tile) in map.tiles.iter().enumerate() {
        // Render a tile depending on the tile type
        if map.revealed_tiles[idx] {
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
        }

        // Next coord
        x += 1;
        if x > map.width - 1 {
            x = 0;
            y += 1;
        }
    }
}
