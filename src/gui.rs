use rltk::{Point, Rltk, RGB};
use specs::prelude::*;

use crate::{CombatStats, GameLog, Map, Name, Player, Position};

/// Draw the UI onto the game screen.
pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    // Draw borders of console at bottom of screen, under the map
    ctx.draw_box(
        0,
        43,
        79,
        6,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    // Display as many log messages as we can fit
    let log = ecs.fetch::<GameLog>();
    let mut y = 44;
    for s in log.entries.iter().rev() {
        if y < 49 {
            ctx.print(2, y, s);
        }
        y += 1;
    }

    // Draw the player's health bar on the top-right border of the console
    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    for (_player, stats) in (&players, &combat_stats).join() {
        let health_str = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(
            12,
            43,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            &health_str,
        );

        ctx.draw_bar_horizontal(
            28,
            43,
            51,
            stats.hp,
            stats.max_hp,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK),
        );
    }

    // Draw mouse cursor on top of EVERYTHING
    let (mouse_x, mouse_y) = ctx.mouse_pos();
    ctx.set_bg(mouse_x, mouse_y, RGB::named(rltk::MAGENTA));

    // Draw mouse tooltips on top of that
    draw_tooltips(ecs, ctx);
}

/// Draw tooltips depending on what the mouse is hovering over.
fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let (mouse_x, mouse_y) = ctx.mouse_pos();
    if mouse_x >= map.width || mouse_y >= map.height {
        return;
    }

    let mut tooltip: Vec<String> = Vec::new();
    for (name, position) in (&names, &positions).join() {
        let idx = map.xy_idx(position.x, position.y);
        if position.x == mouse_x && position.y == mouse_y && map.visible_tiles[idx] {
            tooltip.push(name.to_string());
        }
    }

    let fg = RGB::named(rltk::WHITE);
    let bg = RGB::named(rltk::DIM_GREY);

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            width = width.max(s.len() as _);
        }
        width += 3;

        if mouse_x > 40 {
            let arrow_pos = Point::new(mouse_x - 2, mouse_y);
            let left_x = mouse_x - width;
            let mut y = mouse_y;
            for s in tooltip.iter() {
                ctx.print_color(left_x, y, fg, bg, s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x - i, y, fg, bg, " ");
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, fg, bg, "-→");
        } else {
            let arrow_pos = Point::new(mouse_x + 1, mouse_y);
            let left_x = mouse_x + 3;
            let mut y = mouse_y;
            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, fg, bg, s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x + 1 + i, y, fg, bg, " ");
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, fg, bg, "←-");
        }
    }
}
