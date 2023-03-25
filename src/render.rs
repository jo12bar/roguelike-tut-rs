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
                    glyph = rltk::to_cp437('#');
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
