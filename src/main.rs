mod components;
mod damage_system;
mod gamelog;
mod gui;
mod inventory_system;
mod map;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_system;
mod player;
mod rect;
mod render;
mod saveload_system;
mod spawner;
mod visibility_system;

pub use self::components::*;
pub use self::damage_system::DamageSystem;
pub use self::gamelog::GameLog;
pub use self::inventory_system::*;
pub use self::map::*;
pub use self::map_indexing_system::MapIndexingSystem;
pub use self::melee_combat_system::MeleeCombatSystem;
pub use self::monster_ai_system::MonsterAI;
pub use self::player::*;
pub use self::rect::Rect;
pub use self::visibility_system::VisibilitySystem;

use color_eyre::eyre::Context;
use rltk::{GameState, Rltk, RltkBuilder};
use specs::prelude::*;
use specs::saveload::SimpleMarkerAllocator;

/// Set this to `true` to show the entire map and all entities in it,
/// regardless of what's actually visible. Tooltips and such should work
/// long-range too.
pub const DEBUG_MAP_VIEW: bool = cfg!(feature = "debug-map-view");

/// The game is either "Running" or "Waiting for Input."
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    /// Show the item-targeting UI
    ShowTargeting {
        /// The item's range
        range: i32,
        /// A reference to the item entity
        item: Entity,
    },
    /// Show the main menu.
    MainMenu {
        menu_selection: gui::MainMenuSelection,
    },
    SaveGame,
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

        let mut pickup_items = ItemCollectionSystem;
        pickup_items.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem;
        drop_items.run_now(&self.ecs);
        let mut use_potions = ItemUseSystem;
        use_potions.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        // Tick the ECS (or don't) depending on the current runstate. Make sure
        // to transition to a new runstate after doing so.
        let mut new_runstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            new_runstate = *runstate;
        }

        // Only actually draw the main view if we're not on the main menu.
        if !matches!(new_runstate, RunState::MainMenu { .. }) {
            // Render the map
            render::draw_map(&self.ecs, ctx);

            // Render any entity that has a position
            render::draw_entities(&self.ecs, ctx);

            // Draw the GUI on top of everything
            gui::draw_ui(&self.ecs, ctx);
        }

        match new_runstate {
            RunState::MainMenu { .. } => match gui::main_menu(self, ctx) {
                gui::MainMenuResult::NoSelection(cur_selection) => {
                    new_runstate = RunState::MainMenu {
                        menu_selection: cur_selection,
                    }
                }
                gui::MainMenuResult::Selected(selected) => match selected {
                    gui::MainMenuSelection::NewGame => new_runstate = RunState::PreRun,
                    gui::MainMenuSelection::LoadGame => {
                        saveload_system::load_game(&mut self.ecs)
                            .wrap_err("Failed to load game")
                            .unwrap();
                        new_runstate = RunState::AwaitingInput;
                    }
                    gui::MainMenuSelection::Quit => {
                        std::process::exit(0);
                    }
                },
            },

            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs)
                    .wrap_err("Failed to save game")
                    .unwrap();

                new_runstate = RunState::MainMenu {
                    menu_selection: gui::MainMenuSelection::LoadGame,
                };
            }

            RunState::PreRun => {
                self.run_systems();
                new_runstate = RunState::AwaitingInput;
            }

            RunState::AwaitingInput => {
                new_runstate = player_input(self, ctx);
            }

            RunState::PlayerTurn => {
                self.run_systems();
                new_runstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                new_runstate = RunState::AwaitingInput;
            }

            RunState::ShowInventory => match gui::show_inventory(self, ctx) {
                gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                gui::ItemMenuResult::NoResponse => {}
                gui::ItemMenuResult::Selected(item_entity) => {
                    let ranged_items = self.ecs.read_storage::<Ranged>();
                    if let Some(ranged_item) = ranged_items.get(item_entity) {
                        new_runstate = RunState::ShowTargeting {
                            range: ranged_item.range,
                            item: item_entity,
                        };
                    } else {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent
                            .insert(
                                **self.ecs.fetch::<PlayerEntity>(),
                                WantsToUseItem {
                                    item: item_entity,
                                    target: None,
                                },
                            )
                            .expect("Unable to insert intent WantsToUseItem for player");
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            },

            RunState::ShowDropItem => match gui::drop_item_menu(self, ctx) {
                gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                gui::ItemMenuResult::NoResponse => {}
                gui::ItemMenuResult::Selected(item_entity) => {
                    let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                    intent
                        .insert(
                            **self.ecs.fetch::<PlayerEntity>(),
                            WantsToDropItem { item: item_entity },
                        )
                        .expect("Unable to insert intent WantsToDropItem for player");
                    new_runstate = RunState::PlayerTurn;
                }
            },

            RunState::ShowTargeting { range, item } => match gui::ranged_target(self, ctx, range) {
                gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                gui::ItemMenuResult::NoResponse => {}
                gui::ItemMenuResult::Selected(target) => {
                    let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                    intent.insert(**self.ecs.fetch::<PlayerEntity>(), WantsToUseItem { item, target: Some(target) })
                            .expect("Unable to insert intent WantsToUseItem for player after selecting target");
                    new_runstate = RunState::PlayerTurn;
                }
            },
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_runstate;
        }
        damage_system::delete_the_dead(&mut self.ecs);
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
        .with_title("Rust Roguelike")
        .with_fps_cap(60.0)
        .with_fitscreen(true)
        .build()?;
    context.with_post_scanlines(true);
    context.with_mouse_visibility(false);

    let mut gs = State::default();

    components::register_all_components(&mut gs.ecs);

    let mut rng = rltk::RandomNumberGenerator::new();

    let map = Map::new_map_rooms_and_corridors(&mut rng);
    let (player_x, player_y) = map.rooms[0].center();

    gs.ecs.insert(rng);
    gs.ecs.insert(SimpleMarkerAllocator::<Serializable>::new());

    // Create the player
    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);

    // Add monsters and items to each room (except the starting room)
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs.ecs.insert(map);
    gs.ecs.insert(PlayerPos::new(player_x, player_y));
    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::MainMenu {
        menu_selection: gui::MainMenuSelection::NewGame,
    });
    gs.ecs.insert(GameLog::from(
        vec!["Welcome to Rusty Roguelike".to_string()],
    ));

    rltk::main_loop(context, gs)
}
