use specs::prelude::*;

use crate::{CombatStats, GameLog, Name, SufferDamage, WantsToMelee};

/// A system that handles tracking and applying melee damage to entities every ECS tick.
pub struct MeleeCombatSystem;

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(
        &mut self,
        (entities, mut log, mut wants_to_melee, names, combat_stats, mut inflict_damage): Self::SystemData,
    ) {
        for (_entity, wants_to_melee, name, stats) in
            (&entities, &wants_to_melee, &names, &combat_stats).join()
        {
            if stats.hp > 0 {
                let target_stats = combat_stats.get(wants_to_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_to_melee.target).unwrap();

                    let damage = i32::max(0, stats.power - target_stats.defense);

                    if damage == 0 {
                        log.log(format!("{name} is unable to hurt {target_name}"));
                    } else {
                        log.log(format!("{name} hits {target_name}, for {damage} hp."));
                        SufferDamage::new_damage(
                            &mut inflict_damage,
                            wants_to_melee.target,
                            damage,
                        );
                    }
                }
            }
        }

        wants_to_melee.clear();
    }
}
