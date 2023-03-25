use std::fmt;

use rltk::RGB;
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs::saveload::{ConvertSaveload, Marker, SimpleMarker};
use specs::{Component, ConvertSaveload, Entity};

// Required for specs::ConvertSaveload derive proc_macro
#[allow(deprecated)]
use specs::error::NoError;

pub fn register_all_components(ecs: &mut World) {
    ecs.register::<SimpleMarker<Serializable>>();
    ecs.register::<SerializationHelper>();
    ecs.register::<Position>();
    ecs.register::<Renderable>();
    ecs.register::<Player>();
    ecs.register::<Monster>();
    ecs.register::<Item>();
    ecs.register::<Consumable>();
    ecs.register::<ProvidesHealing>();
    ecs.register::<Ranged>();
    ecs.register::<InflictsDamage>();
    ecs.register::<AreaOfEffect>();
    ecs.register::<Confusion>();
    ecs.register::<InBackpack>();
    ecs.register::<WantsToPickupItem>();
    ecs.register::<WantsToDropItem>();
    ecs.register::<WantsToUseItem>();
    ecs.register::<Name>();
    ecs.register::<Viewshed>();
    ecs.register::<BlocksTile>();
    ecs.register::<CombatStats>();
    ecs.register::<WantsToMelee>();
    ecs.register::<SufferDamage>();
}

/// Indicates that an entity should be serialized when the game is saved.
///
/// # Note
/// This marker struct is _not_ actually a [`Component`]. It's not even registered
/// in the ECS. Instead, [`SimpleMarker<Serializable>`][`specs::saveload::SimpleMarker`]
/// is registered.
pub struct Serializable;

/// A wrapper for serializing & saving things that aren't normally stored
/// as entities in the ECS.
///
/// Used by [`crate::saveload_system::save_game()`].
#[derive(Component, ConvertSaveload, Default, Debug, Clone)]
pub struct SerializationHelper {
    pub map: crate::Map,
}

/// Tracks the location of an entity.
#[derive(Component, ConvertSaveload, Default, Debug, Copy, Clone)]
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
#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    /// Like a z-order. Entities with higher `render_order`s will render underneath
    /// entities with lower `render_order`s.
    pub render_order: i32,
}

impl Default for Renderable {
    fn default() -> Self {
        Self {
            glyph: rltk::to_cp437('█'),
            fg: rltk::RGB::named(rltk::WHITE),
            bg: rltk::RGB::named(rltk::BLACK),
            render_order: 0,
        }
    }
}

/// Indicates that an entity is the Player character.
#[derive(Component, Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub struct Player;

/// Indicates that an entity is a Monster.
#[derive(Component, Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub struct Monster;

/// An item that can be picked up and used.
#[derive(Component, Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub struct Item;

/// Indicates that an item is consumable (i.e. can be used up).
#[derive(Component, Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub struct Consumable;

/// Indicates that an item heals the user.
#[derive(Component, Debug, Default, ConvertSaveload, Clone)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

/// Indicates that an item is "Ranged". This typically means that it can be shot,
/// thrown, and so on.
#[derive(Component, Debug, Default, ConvertSaveload, Clone)]
pub struct Ranged {
    /// How far the entity can be thrown
    pub range: i32,
}

/// Indicates that an item can inflict damage upon another entity.
#[derive(Component, Debug, Default, ConvertSaveload, Clone)]
pub struct InflictsDamage {
    /// How much damage this entity can inflict
    pub damage: i32,
}

/// Indicates that an item has an area of effect within which it acts.
#[derive(Component, Debug, Default, ConvertSaveload, Clone)]
pub struct AreaOfEffect {
    /// The radius within which this entity applies its effect
    pub radius: i32,
}

/// This entity can apply confusion, which makes other entities "Confused" for
/// a few turns - during which time they'll do nothing.
#[derive(Component, Debug, Default, Copy, Clone, ConvertSaveload)]
pub struct Confusion {
    pub turns: i32,
}

/// Entities (such as items) tagged with this are in an entity's backpack.
#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct InBackpack {
    pub owner: Entity,
}

/// Entities tagged with this component are attempting to pick up an [`Item`]
/// and put it into their own backpack this ECS tick.
#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

/// Entities tagged with this component intend to drop an item from their backpack.
#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToDropItem {
    pub item: Entity,
}

/// Entities tagged with this component intend to use an item in their backpack this ECS tick.
#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<rltk::Point>,
}

/// An entity's name.
#[derive(Component, Debug, Default, ConvertSaveload, Clone)]
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
#[derive(Component, Debug, ConvertSaveload, Clone)]
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
#[derive(Component, Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub struct BlocksTile;

/// Statistics influencing an entity's health, attack power, defense, etc.
#[derive(Component, Debug, Default, ConvertSaveload, Clone)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

/// Indicates that an entity wants to attack another entity this ECS tick (via melee).
#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToMelee {
    pub target: Entity,
}

/// The cumulative sum of damage that will be inflicted on an entity this ECS tick.
#[derive(Component, Debug, Default, Clone, ConvertSaveload)]
pub struct SufferDamage {
    pub amount: Vec<i32>,
}

impl SufferDamage {
    /// Add a new damage source to a victim entity's SufferDamage component.
    pub fn new_damage(store: &mut WriteStorage<Self>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = Self {
                amount: vec![amount],
            };
            store.insert(victim, dmg).expect(
                "Unable to insert a brand-new incoming damage list into store for victim entity",
            );
        }
    }
}
