use specs::prelude::*;

use crate::{CombatStats, SufferDamage};

/// Applies damage to entities that are schedules to [`SufferDamage`] this ECS tick.
pub struct DamageSystem;

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, (mut stats, mut damage): Self::SystemData) {
        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }

        damage.clear();
    }
}

/// Delete any entities with 0 HP.
pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();

    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let entities = ecs.entities();

        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                dead.push(entity);
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim)
            .expect("Unable to delete dead (0 HP) entity");
    }
}
