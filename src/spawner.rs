use rltk::{RandomNumberGenerator, RGB};
use rustc_hash::FxHashSet;
use specs::prelude::*;

use crate::{
    BlocksTile, CombatStats, Item, Monster, Name, Player, PlayerEntity, Position, Potion, Rect,
    Renderable, Viewshed, MAPWIDTH,
};

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 2;

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
        .build();
    PlayerEntity(ent)
}

/// Spawns a random monster at a given location. Returns a [`specs::Entity`]
/// reference to the monster.
pub fn random_monster(ecs: &mut World, x: i32, y: i32) -> specs::Entity {
    let roll = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        rng.roll_dice(1, 2)
    };
    match roll {
        1 => spawn_orc(ecs, x, y),
        _ => spawn_goblin(ecs, x, y),
    }
}

/// Fills a room with monsters, items, and other stuff.
pub fn spawn_room(ecs: &mut World, room: &Rect) {
    let mut monster_spawn_points: FxHashSet<usize> = FxHashSet::default();
    let mut item_spawn_points: FxHashSet<usize> = FxHashSet::default();
    monster_spawn_points.reserve(MAX_MONSTERS as _);
    item_spawn_points.reserve(MAX_ITEMS as _);

    // Figure out how many monsters and items to spawn, and where to put them
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();

        // This gives a room a 50% chance of not having any monsters. If it does
        // have monsters, it will have between 1 and MAX_MONSTERS of them.
        let num_monsters = (rng.roll_dice(2, MAX_MONSTERS) - MAX_MONSTERS).max(0);

        // Give a room a 25% chance of having an item. If it does have
        // items, it can have between 1 and MAX_ITEMS of them.
        let num_items = (rng.roll_dice(4, MAX_ITEMS) - 3 * MAX_ITEMS).max(0);

        for _ in 0..num_monsters {
            let mut added = false;
            while !added {
                let x = (room.x1 + 1 + rng.roll_dice(1, i32::abs(room.width() - 1))) as usize;
                let y = (room.y1 + 1 + rng.roll_dice(1, i32::abs(room.height() - 1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !monster_spawn_points.contains(&idx) {
                    monster_spawn_points.insert(idx);
                    added = true;
                }
            }
        }

        for _ in 0..num_items {
            let mut added = false;
            while !added {
                let x = (room.x1 + 1 + rng.roll_dice(1, i32::abs(room.width() - 1))) as usize;
                let y = (room.y1 + 1 + rng.roll_dice(1, i32::abs(room.height() - 1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !item_spawn_points.contains(&idx) {
                    item_spawn_points.insert(idx);
                    added = true;
                }
            }
        }
    }

    // Actually spawn the monsters
    for idx in monster_spawn_points.iter() {
        let x = *idx % MAPWIDTH;
        let y = *idx / MAPWIDTH;
        random_monster(ecs, x as i32, y as i32);
    }

    // Actually spawn the potions
    for idx in item_spawn_points.iter() {
        let x = *idx % MAPWIDTH;
        let y = *idx / MAPWIDTH;
        spawn_health_potion(ecs, x as i32, y as i32);
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
        .build()
}

fn spawn_health_potion(ecs: &mut World, x: i32, y: i32) -> specs::Entity {
    ecs.create_entity()
        .with(Item)
        .with(Potion { heal_amount: 8 })
        .with(Name::from("Health Potion"))
        .with(Position::from((x, y)))
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            render_order: 2,
            ..Default::default()
        })
        .build()
}
