use std::convert::AsRef;

use rltk::{Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use strum::{EnumCount, IntoEnumIterator};

use crate::{
    CombatStats, GameLog, InBackpack, Map, Name, Player, PlayerEntity, PlayerPos, Position, Rect,
    RunState, State, Viewshed, DEBUG_MAP_VIEW, MAPHEIGHT, MAPWIDTH,
};

/// Draw the UI onto the game screen.
pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    let color_bg = RGB::named(rltk::BLACK);
    let color_bg_cursor = RGB::named(rltk::MAGENTA);
    let color_fg = RGB::named(rltk::WHITE);
    let color_fg_accent = RGB::named(rltk::YELLOW);
    let color_fg_health = RGB::named(rltk::RED);

    // Draw borders of console at bottom of screen, under the map
    ctx.draw_box(0, 43, 79, 6, color_fg, color_bg);

    // Display as many log messages as we can fit
    let log = ecs.fetch::<GameLog>();
    let mut y = 44;
    for s in log.entries.iter().rev() {
        if y < 49 {
            ctx.print(2, y, s);
        }
        y += 1;
    }

    // Draw the player's health bar on the top-right border of the console
    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    let map = ecs.fetch::<Map>();
    for (_player, stats) in (&players, &combat_stats).join() {
        let depth = format!("Depth: {}", map.depth);
        ctx.print_color(2, 43, color_fg_accent, color_bg, &depth);

        let health_str = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(12, 43, color_fg_accent, color_bg, &health_str);

        ctx.draw_bar_horizontal(
            28,
            43,
            51,
            stats.hp,
            stats.max_hp,
            color_fg_health,
            color_bg,
        );
    }

    // Draw mouse cursor on top of EVERYTHING
    let (mouse_x, mouse_y) = ctx.mouse_pos();
    ctx.set_bg(mouse_x, mouse_y, color_bg_cursor);

    // Draw mouse tooltips on top of that
    draw_tooltips(ecs, ctx);
}

/// Draw tooltips depending on what the mouse is hovering over.
fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let (mouse_x, mouse_y) = ctx.mouse_pos();
    if mouse_x >= map.width || mouse_y >= map.height {
        return;
    }

    let mut tooltip: Vec<String> = Vec::new();
    for (name, position) in (&names, &positions).join() {
        let idx = map.xy_idx(position.x, position.y);
        if position.x == mouse_x
            && position.y == mouse_y
            && (map.visible_tiles[idx] || DEBUG_MAP_VIEW)
        {
            tooltip.push(name.to_string());
        }
    }

    let fg = RGB::named(rltk::WHITE);
    let bg = RGB::named(rltk::DIM_GREY);

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            width = width.max(s.len() as _);
        }
        width += 3;

        if mouse_x > 40 {
            let arrow_pos = Point::new(mouse_x - 2, mouse_y);
            let left_x = mouse_x - width;
            let mut y = mouse_y;
            for s in tooltip.iter() {
                ctx.print_color(left_x, y, fg, bg, s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x - i, y, fg, bg, " ");
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, fg, bg, "-→");
        } else {
            let arrow_pos = Point::new(mouse_x + 1, mouse_y);
            let left_x = mouse_x + 3;
            let mut y = mouse_y;
            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, fg, bg, s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x + 1 + i, y, fg, bg, " ");
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, fg, bg, "←-");
        }
    }
}

/// Things that can happen when the user does something with the item menu (inventory / backpack).
#[derive(PartialEq, Clone)]
pub enum ItemMenuResult<T: PartialEq + Clone> {
    Cancel,
    NoResponse,
    Selected(T),
}

pub fn show_inventory(gs: &mut State, ctx: &mut Rltk) -> ItemMenuResult<Entity> {
    generic_item_selection_dialogue(gs, ctx, "Inventory", RGB::named(rltk::YELLOW))
}

/// Show a dialogue that allows the player to select an item to drop.
pub fn drop_item_menu(gs: &mut State, ctx: &mut Rltk) -> ItemMenuResult<Entity> {
    generic_item_selection_dialogue(gs, ctx, "Drop which item?", RGB::named(rltk::ORANGE))
}

fn generic_item_selection_dialogue<S: ToString>(
    gs: &mut State,
    ctx: &mut Rltk,
    title: S,
    accent_color: RGB,
) -> ItemMenuResult<Entity> {
    let player_entity = gs.ecs.fetch::<PlayerEntity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    // Figure out how many inventory items the player has
    let inventory = (&backpack, &names)
        .join()
        .filter(|(backpack_item, _)| backpack_item.owner == **player_entity);
    let count = inventory.count();

    // Draw the inventory menu
    const MAP_RECT: Rect = Rect::new(0, 0, MAPWIDTH as _, MAPHEIGHT as _);
    const MENU_WIDTH: i32 = 31;
    const MENU_PADDING: i32 = 1;
    let (cx, cy) = MAP_RECT.center();
    let menu_rect = Rect::new_centered(cx, cy, MENU_WIDTH, (count + 2) as i32 + MENU_PADDING);

    let mut x = menu_rect.x1;
    let mut y = menu_rect.y1;

    ctx.draw_box(
        x,
        y,
        MENU_WIDTH,
        menu_rect.height(),
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        x + 1 + MENU_PADDING,
        y,
        accent_color,
        RGB::named(rltk::BLACK),
        title,
    );
    ctx.print_color(
        x + 1 + MENU_PADDING,
        menu_rect.y2,
        accent_color,
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    x += 1 + MENU_PADDING;
    y += 1 + MENU_PADDING;

    let mut equippable: Vec<Entity> = Vec::with_capacity(count);

    for (j, (entity, _, name)) in (&entities, &backpack, &names)
        .join()
        .filter(|(_, pack_item, _)| pack_item.owner == **player_entity)
        .enumerate()
    {
        ctx.set(
            x,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            x + 1,
            y,
            accent_color,
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(
            x + 2,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(x + 4, y, name.to_string());

        equippable.push(entity);
        y += 1;
    }

    match ctx.key {
        None => ItemMenuResult::NoResponse,
        Some(VirtualKeyCode::Escape) => ItemMenuResult::Cancel,
        Some(key) => {
            let selection = rltk::letter_to_option(key);
            if selection > -1 && selection < count as i32 {
                ItemMenuResult::Selected(equippable[selection as usize])
            } else {
                ItemMenuResult::NoResponse
            }
        }
    }
}

pub fn ranged_target(gs: &mut State, ctx: &mut Rltk, range: i32) -> ItemMenuResult<Point> {
    let player_entity = gs.ecs.fetch::<PlayerEntity>();
    let player_pos = gs.ecs.fetch::<PlayerPos>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(
        5,
        0,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Select target:",
    );

    // Highlight available target cells
    let mut available_cells = Vec::new();
    if let Some(visible) = viewsheds.get(**player_entity) {
        // We have a viewshed!
        for cell in visible.visible_tiles.iter() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(**player_pos, *cell);
            if distance <= range as f32 {
                ctx.set_bg(cell.x, cell.y, RGB::named(rltk::BLUE));
                available_cells.push(cell);
            }
        }
    } else {
        // No viewshed. Just cancel.
        return ItemMenuResult::Cancel;
    }

    // Draw the mouse cursor.
    let (mouse_x, mouse_y) = ctx.mouse_pos();
    let valid_target = available_cells
        .iter()
        .any(|cell| cell.x == mouse_x && cell.y == mouse_y);
    if valid_target {
        ctx.set_bg(mouse_x, mouse_y, RGB::named(rltk::CYAN));
        if ctx.left_click {
            return ItemMenuResult::Selected(Point::new(mouse_x, mouse_y));
        }
    } else {
        ctx.set_bg(mouse_x, mouse_y, RGB::named(rltk::RED));
        if ctx.left_click {
            return ItemMenuResult::Cancel;
        }
    }

    ItemMenuResult::NoResponse
}

/// Possible selection options from the main menu.
#[derive(
    PartialEq,
    Copy,
    Clone,
    Debug,
    strum::Display,
    strum::EnumCount,
    strum::AsRefStr,
    strum::EnumIter,
    num_enum::IntoPrimitive,
    num_enum::TryFromPrimitive,
)]
#[repr(u8)]
pub enum MainMenuSelection {
    #[strum(to_string = "Start new game")]
    NewGame = 0,
    #[strum(to_string = "Load game")]
    LoadGame,
    #[strum(to_string = "Quit")]
    Quit,
}

/// The result of interaction with the main menu.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum MainMenuResult {
    /// Indicates the user switched between different menu items (going up/down),
    /// has gone up a level in the menu or tried to quit (pressed <kbd>Esc</kbd>),
    /// or has genuinely selected nothing somehow.
    NoSelection(MainMenuSelection),
    /// Indicates the user actually selected a menu option.
    Selected(MainMenuSelection),
}

/// Display the main menu and handle input this tick.
pub fn main_menu(gs: &mut State, ctx: &mut Rltk) -> MainMenuResult {
    use MainMenuResult::*;
    use MainMenuSelection::*;

    let save_exists = crate::saveload_system::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();

    let bg_color = RGB::named(rltk::BLACK);
    let title_color = RGB::named(rltk::YELLOW);
    let cur_option_color = RGB::named(rltk::MAGENTA);
    let option_color = RGB::named(rltk::WHITE);

    let mut y = 15;

    ctx.print_color_centered(y, title_color, bg_color, "Rust Roguelike");

    if let RunState::MainMenu {
        menu_selection: selection,
    } = *runstate
    {
        // Display the menu
        y += 9;
        for opt in MainMenuSelection::iter() {
            // Only offer to load the game if a save exists!
            if opt == LoadGame && !save_exists {
                continue;
            }

            let color = if selection == opt {
                cur_option_color
            } else {
                option_color
            };
            ctx.print_color_centered(y, color, bg_color, opt.as_ref());
            y += 1;
        }

        // Handle user input
        match ctx.key {
            // If nothing was pressed, change nothing
            None => NoSelection(selection),

            Some(key) => match key {
                // Quit on escape
                VirtualKeyCode::Escape => NoSelection(Quit),

                // Moving up
                VirtualKeyCode::Up | VirtualKeyCode::K | VirtualKeyCode::Numpad8 => {
                    let cur_sel = selection as u8;
                    let mut new_selection = if cur_sel == 0 {
                        MainMenuSelection::COUNT as u8 - 1
                    } else {
                        cur_sel - 1
                    };
                    // Only select Load Game if a save exists
                    if new_selection == LoadGame as _ && !save_exists {
                        new_selection = if new_selection == 0 {
                            let n = MainMenuSelection::COUNT as u8 - 1;
                            if n == LoadGame as _ {
                                n.saturating_sub(1)
                            } else {
                                n
                            }
                        } else {
                            new_selection - 1
                        };
                    }
                    NoSelection(new_selection.try_into().unwrap())
                }

                // Moving down
                VirtualKeyCode::Down | VirtualKeyCode::J | VirtualKeyCode::Numpad2 => {
                    let cur_sel = selection as u8;
                    let mut new_selection = cur_sel + 1;
                    if new_selection == MainMenuSelection::COUNT as _ {
                        new_selection = 0;
                    }
                    // Only select Load Game if a save exists
                    if new_selection == LoadGame as _ && !save_exists {
                        new_selection += 1;
                        if new_selection == MainMenuSelection::COUNT as _ {
                            new_selection = if 0 == LoadGame as _ { 1 } else { 0 };
                        }
                    }
                    NoSelection(new_selection.try_into().unwrap())
                }

                // Select an option
                VirtualKeyCode::Return => Selected(selection),

                _ => NoSelection(selection),
            },
        }
    } else {
        // If this function is called while not in the MainMenu run state,
        // just return NoSelection with a NewGame option
        NoSelection(NewGame)
    }
}
