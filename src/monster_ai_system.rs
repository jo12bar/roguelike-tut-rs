use rltk::console;
use specs::prelude::*;

use crate::{Map, Monster, Name, PlayerPos, Position, Viewshed};

/// A system that handles a [`Monster`]'s AI.
pub struct MonsterAI;

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadExpect<'a, Map>,
        ReadExpect<'a, PlayerPos>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
    );

    fn run(
        &mut self,
        (map, player_pos, mut viewshed, monster, name, mut position): Self::SystemData,
    ) {
        for (mut viewshed, _monster, name, mut pos) in
            (&mut viewshed, &monster, &name, &mut position).join()
        {
            // If the monster can see the player, it starts moving towards the
            // player while shouting insults.
            if viewshed.visible_tiles.contains(&*player_pos) {
                console::log(format!("{name} shouts insults"));

                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y),
                    map.xy_idx(player_pos.x, player_pos.y),
                    &*map,
                );

                if path.success && path.steps.len() > 1 {
                    pos.x = path.steps[1] as i32 % map.width;
                    pos.y = path.steps[1] as i32 / map.width;
                    viewshed.dirty = true;
                }
            }
        }
    }
}
