use rltk::{Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;

use crate::{
    CombatStats, GameLog, InBackpack, Map, Name, Player, PlayerEntity, PlayerPos, Position, Rect,
    State, Viewshed, DEBUG_MAP_VIEW, MAPHEIGHT, MAPWIDTH,
};

/// Draw the UI onto the game screen.
pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    // Draw borders of console at bottom of screen, under the map
    ctx.draw_box(
        0,
        43,
        79,
        6,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

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
    for (_player, stats) in (&players, &combat_stats).join() {
        let health_str = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(
            12,
            43,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            &health_str,
        );

        ctx.draw_bar_horizontal(
            28,
            43,
            51,
            stats.hp,
            stats.max_hp,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK),
        );
    }

    // Draw mouse cursor on top of EVERYTHING
    let (mouse_x, mouse_y) = ctx.mouse_pos();
    ctx.set_bg(mouse_x, mouse_y, RGB::named(rltk::MAGENTA));

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
