use rltk::RGB;
use specs::prelude::*;
use specs::Component;

/// Tracks the location of an entity.
#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// Provides a CP437 character and fg/bg colors to render an entity with.
#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

/// Indicates that an entity is the Player character.
#[derive(Component, Debug)]
pub struct Player;

/// Describes which tiles are visible to an entity, and what the entity's
/// view range is.
#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
}
