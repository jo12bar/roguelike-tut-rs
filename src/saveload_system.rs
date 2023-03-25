use std::convert::Infallible;
use std::fs::File;
use std::path::Path;

use specs::prelude::*;
use specs::saveload::{
    DeserializeComponents, MarkedBuilder, SerializeComponents, SimpleMarker, SimpleMarkerAllocator,
};

use crate::{components::*, PlayerEntity, PlayerPos};

#[derive(Debug, thiserror::Error)]
pub(crate) enum SaveGameError {
    #[error("Failed to serialize ECS component")]
    Serialization {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Unable to create and/or open `{path}` for writing")]
    FileCreation {
        path: std::path::PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to initialize serializer")]
    SerializerInit {
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr; [ $($typ:ty),* $(,)? ]) => {
        {
        let mut result_vec = Vec::new();
        $(
            let res = SerializeComponents::<Infallible, SimpleMarker<Serializable>>::serialize(
                &($ecs.read_storage::<$typ>(), ),
                &$data.0,
                &$data.1,
                &mut $ser,
            );
            result_vec.push(res.map_err(|e| SaveGameError::Serialization {
                source: Box::new(e)
            }));
        )*
        result_vec.into_iter().collect::<Result<Vec<_>, _>>()
        }
    };
}

/// Save the game to `$PWD/savegame.ron`.
pub(crate) fn save_game(ecs: &mut specs::World) -> Result<(), SaveGameError> {
    // Temporarily add a copy of the Map to the ECS world so that it gets serialized with
    // everything else.
    let map_copy = ecs.get_mut::<crate::map::Map>().unwrap().clone();
    let save_helper = ecs
        .create_entity()
        .with(SerializationHelper { map: map_copy })
        .marked::<SimpleMarker<Serializable>>()
        .build();

    // Actually serialize (need a scope for borrow checker)
    {
        let data = (
            ecs.entities(),
            ecs.read_storage::<SimpleMarker<Serializable>>(),
        );

        let writer = File::create("./savegame.ron").map_err(|e| SaveGameError::FileCreation {
            path: std::path::PathBuf::from("./savegame.ron"),
            source: e,
        })?;
        let mut serializer =
            ron::Serializer::new(writer, None).map_err(|e| SaveGameError::SerializerInit {
                source: Box::new(e),
            })?;

        serialize_individually!(
            ecs, serializer, data;
            [
                Position, Renderable, Player, Viewshed, Monster, Name, BlocksTile, CombatStats,
                SufferDamage, WantsToMelee, Item, Consumable, Ranged, InflictsDamage, AreaOfEffect,
                Confusion, ProvidesHealing, InBackpack, WantsToPickupItem, WantsToUseItem,
                WantsToDropItem, SerializationHelper
            ]
        )?;
    }

    // Remove the temporary map copy.
    ecs.delete_entity(save_helper)
        .expect("Unable to delete temporary copy of map from ECS world (this should never happen)");

    Ok(())
}

/// Returns true if the file `savegame.ron` exists in the current working directory.
pub(crate) fn does_save_exist() -> bool {
    Path::new("./savegame.ron").exists()
}

#[derive(Debug, thiserror::Error)]
pub enum LoadGameError {
    #[error("Error while deserializing save game data")]
    Deserialization {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Unable to read `{path}` for loading game data")]
    OpenFile {
        path: std::path::PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to initialize deserializer")]
    DeserializerInit {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Could not find game map in `{savegame_path}`. The game save may be corrupted.")]
    NoMapFound { savegame_path: std::path::PathBuf },
}

macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $entity_data:expr, $marker_data:expr, $marker_allocator_data:expr; [$($typ:ty),* $(,)?]) => {
        {
            let mut result_vec = Vec::new();

            // Have to create this tuple for ownership reasons
            let mut data = ($entity_data, $marker_data, $marker_allocator_data);

            $(
                let res = DeserializeComponents::<Infallible, _>::deserialize(
                    &mut (&mut $ecs.write_storage::<$typ>(), ),
                    &data.0, // entities
                    &mut data.1, // marker
                    &mut data.2, // allocator
                    &mut $de,
                );
                result_vec.push(res.map_err(|e| LoadGameError::Deserialization { source: Box::new(e) }));
            )*

            result_vec.into_iter().collect::<Result<Vec<_>, _>>()
        }
    };
}

pub(crate) fn load_game(ecs: &mut World) -> Result<(), LoadGameError> {
    // Delete every single entity
    {
        let to_delete = ecs.entities().join().collect::<Vec<_>>();
        for ent in to_delete {
            ecs.delete_entity(ent)
               .expect("Somehow unable to delete an entity created by the current app while trying to load new data...? (This should never happen)");
        }
    }

    // Read the savegame file and deserialize it into the ECS
    let data = std::fs::read_to_string("./savegame.ron").map_err(|e| LoadGameError::OpenFile {
        path: std::path::PathBuf::from("./savegame.ron"),
        source: e,
    })?;
    let mut de =
        ron::Deserializer::from_str(&data).map_err(|e| LoadGameError::DeserializerInit {
            source: Box::new(e),
        })?;

    {
        deserialize_individually!(
            ecs,
            de,
            &ecs.entities(),
            &mut ecs.write_storage::<SimpleMarker<Serializable>>(),
            &mut ecs.write_resource::<SimpleMarkerAllocator<Serializable>>();
            [
                Position, Renderable, Player, Viewshed, Monster, Name, BlocksTile, CombatStats,
                SufferDamage, WantsToMelee, Item, Consumable, Ranged, InflictsDamage, AreaOfEffect,
                Confusion, ProvidesHealing, InBackpack, WantsToPickupItem, WantsToUseItem,
                WantsToDropItem, SerializationHelper
            ]
        )?;
    }

    // Find the map and player to add them to the ECS as resources
    let mut serialization_helper_entity: Option<Entity> = None;
    {
        let entities = ecs.entities();
        let serialization_helpers = ecs.read_storage::<SerializationHelper>();
        let players = ecs.read_storage::<Player>();
        let positions = ecs.read_storage::<Position>();

        // Find all entities with a SerializationHelper component. This
        // contains the level map that was previously deserialized from the
        // save data.
        for (entity, serialization_helper) in (&entities, &serialization_helpers).join() {
            // Found one! Replace the global Map resource with whatever map we found.
            let mut level_map = ecs.write_resource::<crate::map::Map>();
            *level_map = serialization_helper.map.clone();

            // The per-tile entity content vector isn't serialized/deserialized.
            // This will be rebuilt every tick anyways, so just allocate an
            // empty vector in the newly-loaded map.
            level_map.tile_content = vec![Vec::new(); super::map::MAPSIZE];

            // Queue the temporary SerializationHelper entity for deletion.
            serialization_helper_entity = Some(entity);
        }

        // Find the player and the player's position, and add them as resources
        for (entity, _player, position) in (&entities, &players, &positions).join() {
            let mut player_pos = ecs.write_resource::<PlayerPos>();
            *player_pos = PlayerPos(rltk::Point::new(position.x, position.y));

            let mut player_resource = ecs.write_resource::<PlayerEntity>();
            *player_resource = PlayerEntity(entity);
        }
    }
    // Delete the serialization helper entity. If we never found one,
    // then return an error (instead of trying to regenerate the world or something)
    if let Some(ent) = serialization_helper_entity {
        ecs.delete_entity(ent).expect("Somehow unable to delete temporary Map serialization helper entity from ECS even though we found it in the ECS (this should never ever happen)");
    } else {
        return Err(LoadGameError::NoMapFound {
            savegame_path: std::path::PathBuf::from("./savegame.ron"),
        });
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteSaveError {
    #[error("Could not delete saved game at `{path}`")]
    CannotRemove {
        source: std::io::Error,
        path: std::path::PathBuf,
    },
}

/// Delete `savegame.ron` in the current working directory
pub(crate) fn delete_save() -> Result<(), DeleteSaveError> {
    let path = Path::new("savegame.ron");

    if path.exists() {
        std::fs::remove_file(path).map_err(|e| DeleteSaveError::CannotRemove {
            source: e,
            path: std::path::PathBuf::from(path),
        })?;
    }

    Ok(())
}
