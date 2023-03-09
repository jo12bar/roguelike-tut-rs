mod components;
mod map;
mod player;
mod rect;
mod render;
mod visibility_system;

pub use self::components::*;
pub use self::map::*;
pub use self::player::*;
pub use self::rect::*;
pub use self::render::*;
pub use self::visibility_system::*;

use rltk::{GameState, Rltk, RltkBuilder, RGB};
use specs::prelude::*;

/// Global game state.
pub struct State {
    ecs: World,
}

impl Default for State {
    fn default() -> Self {
        Self { ecs: World::new() }
    }
}

impl State {
    /// Runs all ECS systems for one ECS tick.
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem;
        vis.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(self, ctx); // handle player input
        self.run_systems(); // tick the ECS

        // Render the map
        draw_map(&self.ecs, ctx);

        // Render anything that has a position
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

fn main() -> rltk::BError {
    let context = RltkBuilder::simple80x50()
        .with_title("Hello RLTK World")
        .with_fps_cap(60.0)
        .build()?;

    let mut gs = State::default();

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    gs.ecs.insert(map);

    // Create the player
    gs.ecs
        .create_entity()
        .with(Position::from((player_x, player_y)))
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            ..Default::default()
        })
        .with(Player)
        .with(Viewshed {
            range: 8,
            ..Default::default()
        })
        .build();

    rltk::main_loop(context, gs)
}
