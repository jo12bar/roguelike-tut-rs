use std::{
    cmp::{max, min},
    ops::{Deref, DerefMut},
};

use rltk::{Rltk, VirtualKeyCode};
use specs::prelude::*;

use crate::{Map, Player, Position, RunState, State, TileType, Viewshed};

/// The player's position. Just a newtype wrapper over a [`rltk::Point`].
///
/// Allows for unambiguously storing the player position as a specs resource.
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct PlayerPos(pub rltk::Point);

impl PlayerPos {
    pub const fn new(x: i32, y: i32) -> Self {
        Self(rltk::Point { x, y })
    }

    /// Update the tracked player position with new (x, y) coords
    pub fn update(&mut self, x: i32, y: i32) {
        self.0.x = x;
        self.0.y = y;
    }
}

impl Deref for PlayerPos {
    type Target = rltk::Point;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PlayerPos {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

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

            // Update the player position resource
            let mut ppos = ecs.write_resource::<PlayerPos>();
            ppos.update(pos.x, pos.y);
        }
    }
}

/// Handle player input.
pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        // Nothing happened
        None => {
            return RunState::Paused;
        }

        // A key was pressed!
        Some(key) => match key {
            // Movement
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

            // We don't care about this key
            _ => {
                return RunState::Paused;
            }
        },
    }

    RunState::Running
}
