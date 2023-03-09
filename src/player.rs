use std::cmp::{max, min};

use rltk::{Rltk, VirtualKeyCode};
use specs::prelude::*;

use crate::{Map, Player, Position, State, TileType, Viewshed};

/// Try to move the player by a certain delta vector, if the ECS contains
/// at least one entity that has both the [`Position`] and [`Player`] components.
///
/// Will prevent the player from moving off-screen or through walls.
pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let map = ecs.fetch::<Map>();

    for (_player, pos, viewshed) in (&mut players, &mut positions, &mut viewsheds).join() {
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map.tiles[destination_idx] != TileType::Wall {
            pos.x = min(map.width - 1, max(0, pos.x + delta_x));
            pos.y = min(map.height - 1, max(0, pos.y + delta_y));

            // need to update the viewshed if the player moved somewhere!
            viewshed.dirty = true;
        }
    }
}

/// Handle player input.
pub fn player_input(gs: &mut State, ctx: &mut Rltk) {
    // Player movement
    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Left
            | VirtualKeyCode::A
            | VirtualKeyCode::H
            | VirtualKeyCode::Numpad4 => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right
            | VirtualKeyCode::D
            | VirtualKeyCode::L
            | VirtualKeyCode::Numpad6 => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up
            | VirtualKeyCode::W
            | VirtualKeyCode::K
            | VirtualKeyCode::Numpad8 => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down
            | VirtualKeyCode::S
            | VirtualKeyCode::J
            | VirtualKeyCode::Numpad2 => try_move_player(0, 1, &mut gs.ecs),

            _ => {}
        },
    }
}
