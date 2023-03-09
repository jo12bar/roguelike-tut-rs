use rltk::{console, field_of_view, Point};
use specs::prelude::*;

use crate::{Map, Monster, Name, PlayerPos, Position, Viewshed};

/// A system that handles a [`Monster`]'s AI.
pub struct MonsterAI;

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadExpect<'a, PlayerPos>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, (player_pos, viewshed, monster, name): Self::SystemData) {
        for (viewshed, _monster, name) in (&viewshed, &monster, &name).join() {
            if viewshed.visible_tiles.contains(&*player_pos) {
                console::log(format!("{name} shouts insults"));
            }
        }
    }
}
