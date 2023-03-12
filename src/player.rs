use std::{
    cmp::{max, min},
    ops::{Deref, DerefMut},
};

use rltk::{Rltk, VirtualKeyCode};
use specs::prelude::*;

use crate::{
    CombatStats, GameLog, Item, Map, Player, Position, RunState, State, Viewshed, WantsToMelee,
    WantsToPickupItem,
};

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

/// The player [`specs::Entity`].
///
/// Just a newtype wrapper over a [`specs::Entity`]. Allows for unambiguously
/// storing the player position as a specs resource.
#[derive(Copy, Clone, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct PlayerEntity(pub specs::Entity);

impl From<specs::Entity> for PlayerEntity {
    fn from(entity: specs::Entity) -> Self {
        Self(entity)
    }
}

impl Deref for PlayerEntity {
    type Target = specs::Entity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PlayerEntity {
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
    let combat_stats = ecs.read_storage::<CombatStats>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let map = ecs.fetch::<Map>();

    for (entity, _player, pos, viewshed) in
        (&entities, &mut players, &mut positions, &mut viewsheds).join()
    {
        // bounds check
        if pos.x + delta_x < 1
            || pos.x + delta_x > map.width - 1
            || pos.y + delta_y < 1
            || pos.y + delta_y > map.height - 1
        {
            return;
        }

        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        // Check if there's anything to attack in the tile we're trying to move into
        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                // Found a target! Attack it.
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *potential_target,
                        },
                    )
                    .expect("Player failed to add attack target");
                return; // avoid moving post-attack
            }
        }

        // Move if not blocked
        if !map.blocked[destination_idx] {
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
            return RunState::AwaitingInput;
        }

        // A key was pressed!
        Some(key) => match key {
            // Movement in cardinal directions
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

            // Movement in diagonal directions
            VirtualKeyCode::Numpad9 | VirtualKeyCode::I => try_move_player(1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad7 | VirtualKeyCode::U => try_move_player(-1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad3 | VirtualKeyCode::M => try_move_player(1, 1, &mut gs.ecs),

            VirtualKeyCode::Numpad1 | VirtualKeyCode::N => try_move_player(-1, 1, &mut gs.ecs),

            // Item manipulation
            VirtualKeyCode::G => get_item(&mut gs.ecs),
            VirtualKeyCode::B => return RunState::ShowInventory,

            // We don't care about this key
            _ => {
                return RunState::AwaitingInput;
            }
        },
    }

    RunState::PlayerTurn
}

/// Let the player pick up an item.
fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<PlayerPos>();
    let player_entity = ecs.fetch::<PlayerEntity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog.log("There is nothing here to pick up."),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(
                    **player_entity,
                    WantsToPickupItem {
                        collected_by: **player_entity,
                        item,
                    },
                )
                .expect("Unable to insert `WantsToPickupItem` intent for player entity");
        }
    }
}
