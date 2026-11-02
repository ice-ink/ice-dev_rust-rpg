//! # `engine` — the ICE RPG core
//!
//! This crate holds all the *rules* of the game world and no rendering or
//! windowing code. Keeping the rules here (instead of in the Bevy app) means
//! we can test them with a plain `cargo test`, and a learner can read them
//! without also having to understand a game engine at the same time.
//!
//! The crate is split into small, focused modules:
//!
//! - [`map`] — terrain tiles and the 2D tile grid.
//! - [`position`] — a cell on the map, and the mapping to screen space.
//! - [`entity`] — a thing in the world (the player now; NPCs/enemies later).
//! - [`world`] — the whole world: a map plus its entities, and movement.
//!
//! The spatial work (grid storage, coordinate arithmetic) is delegated to the
//! [`gametools`](https://docs.rs/gametools) crate, which we chose as our
//! "grid, pathfinding and field-of-view" backbone. See `v0_01/v0_01.md` for
//! the design notes.

pub mod entity;
pub mod map;
pub mod position;
pub mod world;

pub use entity::{Entity, EntityId};
pub use map::{Map, MapError, TileKind};
pub use position::Position;
pub use world::{World, WorldError};

// Re-export the two `gametools` types that appear in our public API, so callers
// (the `game` crate) only need to depend on `engine`, not on `gametools`.
pub use gametools::{Point, PointDelta};
