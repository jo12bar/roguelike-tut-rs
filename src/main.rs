mod components;
mod damage_system;
mod gamelog;
mod gui;
mod map;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_system;
mod player;
mod rect;
mod render;
mod spawner;
mod visibility_system;

pub use self::components::*;
pub use self::damage_system::DamageSystem;
pub use self::gamelog::GameLog;
pub use self::map::*;
pub use self::map_indexing_system::MapIndexingSystem;
pub use self::melee_combat_system::MeleeCombatSystem;
pub use self::monster_ai_system::MonsterAI;
pub use self::player::*;
pub use self::rect::Rect;
pub use self::render::*;
pub use self::visibility_system::VisibilitySystem;

use rltk::{GameState, Rltk, RltkBuilder};
use specs::prelude::*;

/// The game is either "Running" or "Waiting for Input."
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
}

/// Global game state.
pub struct State {
    pub ecs: World,
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

        let mut mob = MonsterAI;
        mob.run_now(&self.ecs);

        let mut mapindex = MapIndexingSystem;
        mapindex.run_now(&self.ecs);

        let mut melee = MeleeCombatSystem;
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem;
        damage.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        // Tick the ECS (or don't) depending on the current runstate. Make sure
        // to transition to a new runstate after doing so.
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        damage_system::delete_the_dead(&mut self.ecs);

        // Render the map
        draw_map(&self.ecs, ctx);

        // Render any entity that has a position
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            // Only render the entity if the player can currently see it!
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }

        // Draw the GUI on top of everything
        gui::draw_ui(&self.ecs, ctx);
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    run_game().map_err(RunGameError::from)?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
#[error("Error while running game")]
struct RunGameError {
    #[from]
    source: Box<dyn std::error::Error + Send + Sync>,
}

fn run_game() -> rltk::BError {
    let mut context = RltkBuilder::simple80x50()
        .with_title("Hello RLTK World")
        .with_fps_cap(60.0)
        .with_fitscreen(true)
        .build()?;
    context.with_post_scanlines(true);
    context.with_mouse_visibility(false);

    let mut gs = State::default();

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();

    let mut rng = rltk::RandomNumberGenerator::new();

    let map = Map::new_map_rooms_and_corridors(&mut rng);
    let (player_x, player_y) = map.rooms[0].center();

    gs.ecs.insert(rng);

    // Create the player
    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);

    // Add monsters to the center of each room (except the starting room)
    for room in map.rooms.iter().skip(1) {
        let (x, y) = room.center();
        spawner::random_monster(&mut gs.ecs, x, y);
    }

    gs.ecs.insert(map);
    gs.ecs.insert(PlayerPos::new(player_x, player_y));
    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(GameLog::from(
        vec!["Welcome to Rusty Roguelike".to_string()],
    ));

    rltk::main_loop(context, gs)
}
