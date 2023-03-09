use specs::prelude::*;

use crate::{BlocksTile, Map, Position};

/// A system that continually keeps track of things like blocked tiles in the
/// current map.
pub struct MapIndexingSystem;

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
    );

    fn run(&mut self, (mut map, position, blockers): Self::SystemData) {
        // Update statically-blocked tiles in blocked index. Also has the effect
        // of un-blocking tiles that were previously blocked by a moving entity.
        map.populate_blocked();

        for (position, _blocks) in (&position, &blockers).join() {
            let idx = map.xy_idx(position.x, position.y);
            map.blocked[idx] = true;
        }
    }
}
