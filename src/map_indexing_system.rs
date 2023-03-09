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
        Entities<'a>,
    );

    fn run(&mut self, (mut map, position, blockers, entities): Self::SystemData) {
        // Update statically-blocked tiles in blocked index. Also has the effect
        // of un-blocking tiles that were previously blocked by a moving entity.
        map.populate_blocked();

        // Clear out the previous tick's tile content index.
        map.clear_content_index();

        // Iterate all entities with postitions.
        for (entity, position) in (&entities, &position).join() {
            let idx = map.xy_idx(position.x, position.y);

            // If they block this tile from other entities, add to the blocking list.
            let _p: Option<&BlocksTile> = blockers.get(entity);
            if let Some(_p) = _p {
                map.blocked[idx] = true;
            }

            // Push the entity to the appropriate tile content index slot.
            map.tile_content[idx].push(entity);
        }
    }
}
