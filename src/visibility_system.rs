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
            //viewshed.visible_tiles.clear();
            viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
            viewshed.visible_tiles.retain(|p| map.in_bounds(*p));

            // If this is the player, reveal what they can see!
            if let Some(_p) = player.get(ent) {
                for vis in viewshed.visible_tiles.iter() {
                    let idx = map.xy_idx(vis.x, vis.y);
                    map.revealed_tiles[idx] = true;
                }
            }
        }
    }
}
