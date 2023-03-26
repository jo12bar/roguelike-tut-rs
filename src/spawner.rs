use std::collections::hash_map;

use rltk::{RandomNumberGenerator, RGB};
use rustc_hash::FxHashMap;
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};

use crate::rng_table::RngTable;
use crate::{
    AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, InflictsDamage, Item, Monster,
    Name, Player, PlayerEntity, Position, ProvidesHealing, Ranged, Rect, Renderable, Serializable,
    Viewshed, MAPWIDTH,
};

const SPAWN_DIE: i32 = 7;
const MAX_SPAWN_TRIES_PER_ROOM: usize = 20;

/// Spawns the player and returns their [`PlayerEntity`] reference.
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> PlayerEntity {
    let ent = ecs
        .create_entity()
        .with(Player)
        .with(Name::from("Player"))
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .with(Position::from((player_x, player_y)))
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            render_order: 0,
            ..Default::default()
        })
        .with(Viewshed {
            range: 8,
            ..Default::default()
        })
        .marked::<SimpleMarker<Serializable>>()
        .build();
    PlayerEntity(ent)
}

fn room_entity_spawn_table(map_depth: i32) -> RngTable {
    RngTable::new()
        .add("Goblin", 10)
        .add("Orc", 1 + map_depth)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2 + map_depth)
        .add("Confusion Scroll", 2 + map_depth)
        .add("Magic Missile Scroll", 4)
}

/// Fills a room with monsters, items, and other stuff.
pub fn spawn_room(ecs: &mut World, room: &Rect, map_depth: i32) {
    let spawn_table = room_entity_spawn_table(map_depth);
    let mut spawn_points: FxHashMap<usize, Option<String>> = FxHashMap::default();

    // Figure out how many monsters and items to spawn, and where to put them
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();

        // This gives a room a spawn count following the roll of 1d(SPAWN_DIE) - floor(SPAWN_DIE / 2),
        // plus 1 for each level past the first floor.
        let num_spawns = rng.roll_dice(1, SPAWN_DIE + (SPAWN_DIE as f32 / 2.0).floor() as i32)
            + (map_depth - 1)
            - (SPAWN_DIE as f32 / 2.0).floor() as i32;

        for _ in 0..num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < MAX_SPAWN_TRIES_PER_ROOM {
                let x = (room.x1 + 1 + rng.roll_dice(1, i32::abs(room.width() - 1))) as usize;
                let y = (room.y1 + 1 + rng.roll_dice(1, i32::abs(room.height() - 1))) as usize;
                let idx = (y * MAPWIDTH) + x;

                if let hash_map::Entry::Vacant(e) = spawn_points.entry(idx) {
                    e.insert(spawn_table.roll(&mut rng).map(|s| s.to_string()));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    // Actually spawn the entities
    for (map_idx, roll_result) in spawn_points.iter() {
        let x = (*map_idx % MAPWIDTH) as i32;
        let y = (*map_idx / MAPWIDTH) as i32;

        if let Some(roll_result) = roll_result {
            match roll_result.as_ref() {
                "Goblin" => spawn_goblin(ecs, x, y),
                "Orc" => spawn_orc(ecs, x, y),
                "Health Potion" => spawn_health_potion(ecs, x, y),
                "Fireball Scroll" => spawn_fireball_scroll(ecs, x, y),
                "Confusion Scroll" => spawn_confusion_scroll(ecs, x, y),
                "Magic Missile Scroll" => spawn_magic_missile_scroll(ecs, x, y),
                s => unreachable!("Should be impossible to roll entity {s:?} that isn't in the spawn table, but here we are!"),
            };
        }
    }
}

fn spawn_orc(ecs: &mut World, x: i32, y: i32) -> specs::Entity {
    spawn_monster(ecs, x, y, rltk::to_cp437('o'), "Orc")
}

fn spawn_goblin(ecs: &mut World, x: i32, y: i32) -> specs::Entity {
    spawn_monster(ecs, x, y, rltk::to_cp437('g'), "Goblin")
}

fn spawn_monster<S: ToString>(
    ecs: &mut World,
    x: i32,
    y: i32,
    glyph: rltk::FontCharType,
    name: S,
) -> specs::Entity {
    ecs.create_entity()
        .with(Monster)
        .with(Name::from(name.to_string()))
        .with(BlocksTile)
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .with(Position::from((x, y)))
        .with(Renderable {
            glyph,
            fg: RGB::named(rltk::RED),
            render_order: 1,
            ..Default::default()
        })
        .with(Viewshed {
            range: 8,
            ..Default::default()
        })
        .marked::<SimpleMarker<Serializable>>()
        .build()
}

fn spawn_health_potion(ecs: &mut World, x: i32, y: i32) -> specs::Entity {
    ecs.create_entity()
        .with(Item)
        .with(Consumable)
        .with(ProvidesHealing { heal_amount: 8 })
        .with(Name::from("Health Potion"))
        .with(Position::from((x, y)))
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            render_order: 2,
            ..Default::default()
        })
        .marked::<SimpleMarker<Serializable>>()
        .build()
}

fn spawn_fireball_scroll(ecs: &mut World, x: i32, y: i32) -> specs::Entity {
    ecs.create_entity()
        .with(Item)
        .with(Consumable)
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 20 })
        .with(AreaOfEffect { radius: 3 })
        .with(Name::from("Fireball Scroll"))
        .with(Position::from((x, y)))
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::ORANGE),
            render_order: 2,
            ..Default::default()
        })
        .marked::<SimpleMarker<Serializable>>()
        .build()
}

fn spawn_magic_missile_scroll(ecs: &mut World, x: i32, y: i32) -> specs::Entity {
    ecs.create_entity()
        .with(Item)
        .with(Consumable)
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 8 })
        .with(Name::from("Magic Missile Scroll"))
        .with(Position::from((x, y)))
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN),
            render_order: 2,
            ..Default::default()
        })
        .marked::<SimpleMarker<Serializable>>()
        .build()
}

fn spawn_confusion_scroll(ecs: &mut World, x: i32, y: i32) -> specs::Entity {
    ecs.create_entity()
        .with(Item)
        .with(Consumable)
        .with(Ranged { range: 6 })
        .with(Confusion { turns: 4 })
        .with(Name::from("Confusion Scroll"))
        .with(Position::from((x, y)))
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::PINK),
            render_order: 2,
            ..Default::default()
        })
        .marked::<SimpleMarker<Serializable>>()
        .build()
}
