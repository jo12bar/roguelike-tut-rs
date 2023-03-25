use rltk::{field_of_view, Algorithm2D, Point};
use specs::prelude::*;

use crate::{Map, Player, Position, Viewshed};

/// A system that updates the visible tiles for any entity with a [`Viewshed`]
/// and a [`Position`].
pub struct VisibilitySystem;

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, (mut map, entities, mut viewshed, pos, player): Self::SystemData) {
        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.dirty = false;

                //viewshed.visible_tiles.clear();
                viewshed.visible_tiles =
                    field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
                viewshed.visible_tiles.retain(|p| map.in_bounds(*p));

                // If this is the player, reveal what they can see!
                if let Some(_p) = player.get(ent) {
                    // Grey out all tiles that were visible to the player the last time the
                    // viewshed was updated.
                    for mut t in map.visible_tiles.iter_mut() {
                        *t = false;
                    }

                    // Update the map's record of currently-visible tiles and
                    // previously-revelaed tiles.
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);
                        map.revealed_tiles.set(idx, true);
                        map.visible_tiles.set(idx, true);
                    }
                }
            }
        }
    }
}
