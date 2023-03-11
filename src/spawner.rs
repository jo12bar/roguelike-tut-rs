use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;

use crate::{
    BlocksTile, CombatStats, Monster, Name, Player, PlayerEntity, Position, Renderable, Viewshed,
};

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
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed {
            range: 8,
            ..Default::default()
        })
        .build()
}
