use specs::prelude::*;

use crate::{
    AreaOfEffect, CombatStats, Confusion, Consumable, GameLog, InBackpack, InflictsDamage, Map,
    Name, PlayerEntity, Position, ProvidesHealing, SufferDamage, WantsToDropItem,
    WantsToPickupItem, WantsToUseItem,
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
        ReadExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Confusion>,
        ReadStorage<'a, Consumable>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(
        &mut self,
        (
            player_entity,
            mut gamelog,
            map,
            entities,
            mut wants_use_item,
            names,
            healing,
            damage_inflictors,
            areas_of_effect,
            mut confused,
            consumables,
            mut combat_stats,
            mut suffer_damage,
        ): Self::SystemData,
    ) {
        for (entity, use_item) in (&entities, &wants_use_item).join() {
            let mut used_item = false;

            // Targeting
            let mut targets = Vec::new();
            if let Some(target) = use_item.target {
                if let Some(aoe) = areas_of_effect.get(use_item.item) {
                    // Item has an area of effect. Figure out which cells to target.
                    let blast_cells = rltk::field_of_view(target, aoe.radius, &*map);
                    for cell in blast_cells.iter().filter(|p| {
                        p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1
                    }) {
                        let idx = map.xy_idx(cell.x, cell.y);
                        for mob in map.tile_content[idx].iter() {
                            targets.push(*mob);
                        }
                    }
                } else {
                    // Assume single-tile target.
                    let idx = map.xy_idx(target.x, target.y);
                    for mob in map.tile_content[idx].iter() {
                        targets.push(*mob);
                    }
                }
            } else {
                // Target the item user by default
                targets.push(entity);
            }

            // If it inflicts damage, apply it to the target cell
            if let Some(damager) = damage_inflictors.get(use_item.item) {
                used_item = false;
                for mob in targets.iter() {
                    SufferDamage::new_damage(&mut suffer_damage, *mob, damager.damage);
                    if *player_entity == entity {
                        let mob_name = names.get(*mob).unwrap();
                        let item_name = names.get(use_item.item).unwrap();
                        gamelog.log(format!(
                            "You use {item_name} on {mob_name}, inflicting {} hp.",
                            damager.damage
                        ));
                    }

                    used_item = true;
                }
            }

            // If the item provides healing, apply the healing.
            if let Some(healer) = healing.get(use_item.item) {
                used_item = false;

                for target in targets.iter() {
                    if let Some(stats) = combat_stats.get_mut(*target) {
                        stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                        if *player_entity == entity {
                            gamelog.log(format!(
                                "You drink the {}, healing {} hp.",
                                names.get(use_item.item).unwrap(),
                                healer.heal_amount
                            ));
                        }
                        used_item = true;
                    }
                }
            }

            // If the item confuses entities, it's time to absolutely just outright blow their
            // minds with the pure confusion
            if let Some(confusion) = confused.get(use_item.item).copied() {
                used_item = false;
                for mob in targets.iter() {
                    if *player_entity == entity {
                        let mob_name = names.get(*mob).unwrap();
                        let item_name = names.get(use_item.item).unwrap();
                        gamelog.log(format!(
                            "You use {item_name} on {mob_name}, confusing them."
                        ));
                    }

                    confused
                        .insert(*mob, confusion)
                        .expect("Unable to insert Confusion component for entity");

                    used_item = true;
                }
            }

            // Delete the item if it's consumable
            if used_item && consumables.get(use_item.item).is_some() {
                entities
                    .delete(use_item.item)
                    .expect("Failed to delete potion entity that just got drank");
            }
        }

        wants_use_item.clear();
    }
}
