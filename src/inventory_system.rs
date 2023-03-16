use specs::prelude::*;

use crate::{
    CombatStats, Consumable, GameLog, InBackpack, Name, PlayerEntity, Position, ProvidesHealing,
    WantsToDropItem, WantsToPickupItem, WantsToUseItem,
};

/// Searches for any entities that [`WantsToPickupItem`] and let's them pick
/// it up and put it into their backpack for later use.
pub struct ItemCollectionSystem;

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, PlayerEntity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(
        &mut self,
        (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack): Self::SystemData,
    ) {
        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to insert backpack entry when entity tried to pick up item");

            if pickup.collected_by == **player_entity {
                gamelog.log(format!(
                    "You pick up the {}.",
                    names.get(pickup.item).unwrap()
                ))
            }
        }

        wants_pickup.clear();
    }
}

/// Whenever an entity [`WantsToDropItem`], remove the item from their inventory and
/// place it at their location in the game world.
pub struct ItemDropSystem;
impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        ReadExpect<'a, PlayerEntity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(
        &mut self,
        (player_entity, mut gamelog, entities, mut wants_drop, names, mut positions, mut backpack): Self::SystemData,
    ) {
        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let dropper_pos = *positions.get(entity).unwrap();

            positions
                .insert(to_drop.item, dropper_pos)
                .expect("Unable to insert dropped item position");
            backpack.remove(to_drop.item);

            if entity == **player_entity {
                gamelog.log(format!(
                    "You drop the {}.",
                    names.get(to_drop.item).unwrap()
                ));
            }
        }

        wants_drop.clear();
    }
}

/// A system that allows entities that [`WantsToUseItem`] to use their item.
pub struct ItemUseSystem;

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = (
        ReadExpect<'a, PlayerEntity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, Consumable>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(
        &mut self,
        (
            player_entity,
            mut gamelog,
            entities,
            mut wants_use_item,
            names,
            healing,
            consumables,
            mut combat_stats,
        ): Self::SystemData,
    ) {
        for (entity, use_item) in (&entities, &wants_use_item).join() {
            // If the item provides healing, heal the user
            if let Some(healer) = healing.get(use_item.item) {
                if let Some(stats) = combat_stats.get_mut(entity) {
                    stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                    if entity == **player_entity {
                        gamelog.log(format!(
                            "You drink the {}, healing {} hp.",
                            names.get(use_item.item).unwrap(),
                            healer.heal_amount
                        ));
                    }
                }
            }

            // Delete the item if it's consumable
            if consumables.get(use_item.item).is_some() {
                entities
                    .delete(use_item.item)
                    .expect("Failed to delete potion entity that just got drank");
            }
        }

        wants_use_item.clear();
    }
}
