use specs::prelude::*;

use crate::{
    CombatStats, GameLog, InBackpack, Name, PlayerEntity, Position, Potion, WantsToDrinkPotion,
    WantsToPickupItem,
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

/// A system that allows entities that [`WantsToDrinkPotion`] to drink their potion.
pub struct PotionUseSystem;

impl<'a> System<'a> for PotionUseSystem {
    type SystemData = (
        ReadExpect<'a, PlayerEntity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDrinkPotion>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Potion>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(
        &mut self,
        (player_entity, mut gamelog, entities, mut wants_drink, names, potions, mut combat_stats): Self::SystemData,
    ) {
        for (entity, drink, stats) in (&entities, &wants_drink, &mut combat_stats).join() {
            let potion = potions.get(drink.potion);
            if let Some(potion) = potion {
                stats.hp = i32::min(stats.max_hp, stats.hp + potion.heal_amount);
                if entity == **player_entity {
                    gamelog.log(format!(
                        "You drink the {}, healing {} hp.",
                        names.get(drink.potion).unwrap(),
                        potion.heal_amount
                    ));
                }
                entities
                    .delete(drink.potion)
                    .expect("Failed to delete potion entity that just got drank");
            }
        }

        wants_drink.clear();
    }
}
