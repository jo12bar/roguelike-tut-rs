use rltk::{Console, Rltk, RGB};
use specs::prelude::*;

use crate::{CombatStats, Player};

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
}
