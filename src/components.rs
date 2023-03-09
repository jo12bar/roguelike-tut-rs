use std::fmt;

use rltk::RGB;
use specs::prelude::*;
use specs::Component;

/// Tracks the location of an entity.
#[derive(Component, Default, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl From<(i32, i32)> for Position {
    fn from((x, y): (i32, i32)) -> Self {
        Self { x, y }
    }
}

/// Provides a CP437 character and fg/bg colors to render an entity with.
#[derive(Component, Debug)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

impl Default for Renderable {
    fn default() -> Self {
        Self {
            glyph: rltk::to_cp437('█'),
            fg: rltk::RGB::named(rltk::WHITE),
            bg: rltk::RGB::named(rltk::BLACK),
        }
    }
}

/// Indicates that an entity is the Player character.
#[derive(Component, Debug, Default)]
pub struct Player;

/// Indicates that an entity is a Monster.
#[derive(Component, Debug, Default)]
pub struct Monster;

/// An entity's name.
#[derive(Component, Debug, Default)]
pub struct Name {
    pub name: String,
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)
    }
}

impl From<String> for Name {
    fn from(name: String) -> Self {
        Self { name }
    }
}

impl<'a> From<&'a str> for Name {
    fn from(name: &'a str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

/// Describes which tiles are visible to an entity, and what the entity's
/// view range is.
#[derive(Component, Debug)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    /// `true` if the viewshed needs to be updated
    pub dirty: bool,
}

impl Default for Viewshed {
    fn default() -> Self {
        Self {
            visible_tiles: Vec::new(),
            range: 4,
            dirty: true,
        }
    }
}

/// Indicates that an entity blocks the tile it is currently on from access by
/// other entities.
#[derive(Component, Debug, Default)]
pub struct BlocksTile;
