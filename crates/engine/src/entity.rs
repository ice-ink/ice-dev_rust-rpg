//! Things that exist in the world: the player now, NPCs and enemies later.

use gametools::{GameResult, MeteredResource};

use crate::position::Position;

/// A unique identifier for an entity within a [`World`](crate::world::World).
///
/// The `u64` inside is just a number you choose when creating the entity; the
/// world will reject duplicates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId(pub u64);

/// Action points granted at the start of a turn. Combat arrives in a later
/// version, but we model the stat now so movement and combat share one concept.
const AP_START: u32 = 10;

/// Maximum hit points. Every entity starts fully healthy.
const HP_MAX: u32 = 10;

/// A single entity in the world.
///
/// `action_points` and `hit_points` are [`MeteredResource`]s — bounded counters
/// that clamp instead of overflowing. We borrow them from `gametools` rather
/// than hand-rolling min/max logic.
#[derive(Debug)]
pub struct Entity {
    id: EntityId,
    name: String,
    position: Position,
    action_points: MeteredResource<u32>,
    hit_points: MeteredResource<u32>,
}

impl Entity {
    /// Creates an entity with full health and a fresh pool of action points.
    ///
    /// Returns a [`GameResult`] because `gametools` validates the resource
    /// bounds; with the constants above it cannot actually fail.
    pub fn new(id: EntityId, name: impl Into<String>, position: Position) -> GameResult<Self> {
        Ok(Self {
            id,
            name: name.into(),
            position,
            action_points: MeteredResource::<u32>::new("action_points", 0, AP_START, AP_START)?,
            hit_points: MeteredResource::<u32>::new("hit_points", 0, HP_MAX, HP_MAX)?,
        })
    }

    /// This entity's unique id.
    pub fn id(&self) -> EntityId {
        self.id
    }

    /// This entity's display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Where this entity currently stands.
    pub fn position(&self) -> Position {
        self.position
    }

    /// Current action points.
    pub fn action_points(&self) -> u32 {
        self.action_points.current()
    }

    /// Current hit points.
    pub fn hit_points(&self) -> u32 {
        self.hit_points.current()
    }

    /// Places this entity at a new cell. Only visible within the engine crate,
    /// because movement must go through [`World::try_move`](crate::world::World::try_move)
    /// so bounds and terrain are always checked.
    pub(crate) fn set_position(&mut self, position: Position) {
        self.position = position;
    }
}
