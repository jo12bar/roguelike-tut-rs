use rltk::{Rltk, RGB};
use specs::World;

use crate::{Map, TileType, DEBUG_MAP_VIEW};

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
                    glyph = rltk::to_cp437('#');
                    fg = RGB::from_f32(0.0, 1.0, 0.0);
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
