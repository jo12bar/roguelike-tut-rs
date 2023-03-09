use rltk::{console, field_of_view, Point};
use specs::prelude::*;

use crate::{Map, Monster, PlayerPos, Position, Viewshed};

/// A system that handles a [`Monster`]'s AI.
pub struct MonsterAI;

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadExpect<'a, PlayerPos>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
    );

    fn run(&mut self, (player_pos, viewshed, monster): Self::SystemData) {
        for (viewshed, _monster) in (&viewshed, &monster).join() {
            if viewshed.visible_tiles.contains(&*player_pos) {
                console::log("Monster considers their own existence");
            }
        }
    }
}
