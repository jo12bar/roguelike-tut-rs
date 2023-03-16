use rltk::Point;
use specs::prelude::*;

use crate::{
    Confusion, Map, Monster, PlayerEntity, PlayerPos, Position, RunState, Viewshed, WantsToMelee,
};

/// A system that handles a [`Monster`]'s AI.
pub struct MonsterAI;

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, PlayerPos>,
        ReadExpect<'a, PlayerEntity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, Confusion>,
    );

    fn run(
        &mut self,
        (
            mut map,
            player_pos,
            player_entity,
            runstate,
            entities,
            mut viewshed,
            monster,
            mut position,
            mut wants_to_melee,
            mut confused,
        ): Self::SystemData,
    ) {
        // Only run when it's the monsters' turn!
        if *runstate != RunState::MonsterTurn {
            return;
        }

        for (entity, mut viewshed, _monster, mut pos) in
            (&entities, &mut viewshed, &monster, &mut position).join()
        {
            // Check if the monster can actually act right now (is it confused, for example?)
            let mut can_act = true;

            if let Some(i_am_confused) = confused.get_mut(entity) {
                i_am_confused.turns -= 1;
                if i_am_confused.turns < 1 {
                    confused.remove(entity);
                }
                can_act = false;
            }

            if can_act {
                // If the monster is close enough, it attacks (and doesn't move).
                let distance = rltk::DistanceAlg::Pythagoras
                    .distance2d(Point::new(pos.x, pos.y), **player_pos);
                if distance < 1.5 {
                    wants_to_melee
                        .insert(
                            entity,
                            WantsToMelee {
                                target: **player_entity,
                            },
                        )
                        .expect(
                            "Monster is unable to insert next attack against player into storage",
                        );
                } else if viewshed.visible_tiles.contains(&*player_pos) {
                    // If the monster can see the player, it starts moving towards the
                    // player.
                    let path = rltk::a_star_search(
                        map.xy_idx(pos.x, pos.y),
                        map.xy_idx(player_pos.x, player_pos.y),
                        &*map,
                    );

                    if path.success && path.steps.len() > 1 {
                        let mut idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = false;
                        pos.x = path.steps[1] as i32 % map.width;
                        pos.y = path.steps[1] as i32 / map.width;
                        idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = true;
                        viewshed.dirty = true;
                    }
                }
            }
        }
    }
}
