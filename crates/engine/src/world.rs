//! The whole world: a [`Map`] plus the entities standing on it, and movement.

use gametools::PointDelta;

use crate::entity::{Entity, EntityId};
use crate::map::Map;
use crate::position::Position;

/// An error returned when moving or spawning an entity fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldError {
    /// No entity with this id exists in the world.
    UnknownEntity(EntityId),
    /// The destination is outside the map.
    OutOfBounds { col: i32, row: i32 },
    /// The destination tile is impassable.
    Blocked { col: i32, row: i32 },
    /// An entity with this id already exists.
    DuplicateEntity(EntityId),
}

impl std::fmt::Display for WorldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorldError::UnknownEntity(id) => write!(f, "no entity with id {id:?}"),
            WorldError::OutOfBounds { col, row } => {
                write!(f, "cannot move to ({col}, {row}): outside the map")
            }
            WorldError::Blocked { col, row } => {
                write!(f, "cannot move to ({col}, {row}): blocked")
            }
            WorldError::DuplicateEntity(id) => write!(f, "entity id {id:?} is already in use"),
        }
    }
}

impl std::error::Error for WorldError {}

/// The world: a map and the entities on it.
pub struct World {
    map: Map,
    entities: Vec<Entity>,
}

impl World {
    /// Creates an empty world (no entities) over the given map.
    pub fn new(map: Map) -> Self {
        Self {
            map,
            entities: Vec::new(),
        }
    }

    /// The map this world is played on.
    pub fn map(&self) -> &Map {
        &self.map
    }

    /// Looks up an entity by id.
    pub fn entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id() == id)
    }

    /// Iterates over every entity in the world.
    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter()
    }

    /// Adds an entity to the world.
    ///
    /// Fails if the id is already taken, or if the entity's starting cell is
    /// not a place it can stand.
    pub fn spawn(&mut self, entity: Entity) -> Result<(), WorldError> {
        if self.entity(entity.id()).is_some() {
            return Err(WorldError::DuplicateEntity(entity.id()));
        }
        let position = entity.position().point();
        if !self.map.is_passable(position) {
            return Err(WorldError::Blocked {
                col: position.col,
                row: position.row,
            });
        }
        self.entities.push(entity);
        Ok(())
    }

    /// Moves an entity one cell in the direction `delta`, if it can.
    ///
    /// The move is rejected (with a descriptive error, leaving the entity where
    /// it is) when the entity is unknown, the destination is outside the map,
    /// or the destination is a wall.
    pub fn try_move(&mut self, id: EntityId, delta: PointDelta) -> Result<(), WorldError> {
        let index = self
            .entities
            .iter()
            .position(|e| e.id() == id)
            .ok_or(WorldError::UnknownEntity(id))?;

        let origin = self.entities[index].position().point();

        // Ask the map where a step in this direction lands. Bounded topology
        // returns `None` when the step would leave the map.
        let destination = match self.map.step(origin, delta) {
            Some(destination) => destination,
            None => {
                let raw = origin + delta;
                return Err(WorldError::OutOfBounds {
                    col: raw.col,
                    row: raw.row,
                });
            }
        };

        if !self.map.is_passable(destination) {
            return Err(WorldError::Blocked {
                col: destination.col,
                row: destination.row,
            });
        }

        self.entities[index].set_position(Position::from(destination));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gametools::PointDelta;

    /// Builds a 5×5 all-floor world with a single entity ("Hero") at (2, 2).
    fn world_with_player() -> (World, EntityId) {
        let mut world = World::new(Map::new(5, 5).unwrap());
        let id = EntityId(1);
        world
            .spawn(Entity::new(id, "Hero", Position::new(2, 2)).unwrap())
            .unwrap();
        (world, id)
    }

    #[test]
    fn spawn_places_entity() {
        let (world, id) = world_with_player();
        let entity = world.entity(id).unwrap();
        assert_eq!(entity.position(), Position::new(2, 2));
        assert_eq!(entity.name(), "Hero");
    }

    #[test]
    fn spawn_rejects_duplicate_id() {
        let (mut world, id) = world_with_player();
        let err = world
            .spawn(Entity::new(id, "Clone", Position::new(3, 3)).unwrap())
            .unwrap_err();
        assert_eq!(err, WorldError::DuplicateEntity(id));
    }

    #[test]
    fn spawn_rejects_wall_position() {
        let mut world = World::new(Map::bordered(5, 5).unwrap());
        let err = world
            .spawn(Entity::new(EntityId(1), "WallHugger", Position::new(0, 0)).unwrap())
            .unwrap_err();
        assert_eq!(err, WorldError::Blocked { col: 0, row: 0 });
    }

    #[test]
    fn move_succeeds_on_floor() {
        let (mut world, id) = world_with_player();
        world.try_move(id, PointDelta::EAST).unwrap();
        assert_eq!(world.entity(id).unwrap().position(), Position::new(3, 2));
    }

    #[test]
    fn move_into_wall_is_blocked() {
        let mut world = World::new(Map::bordered(5, 5).unwrap());
        let id = EntityId(1);
        world
            .spawn(Entity::new(id, "Hero", Position::new(1, 1)).unwrap())
            .unwrap();

        // (1,1) is interior floor; NORTH hits the top wall at (1,0).
        let err = world.try_move(id, PointDelta::NORTH).unwrap_err();
        assert_eq!(err, WorldError::Blocked { col: 1, row: 0 });

        // The entity did not move.
        assert_eq!(world.entity(id).unwrap().position(), Position::new(1, 1));
    }

    #[test]
    fn move_out_of_bounds_is_rejected() {
        let (mut world, id) = world_with_player(); // at (2,2) in a 5×5 floor map
        world.try_move(id, PointDelta::EAST).unwrap(); // -> (3,2)
        world.try_move(id, PointDelta::EAST).unwrap(); // -> (4,2)
        let err = world.try_move(id, PointDelta::EAST).unwrap_err(); // -> (5,2) out of bounds
        assert_eq!(err, WorldError::OutOfBounds { col: 5, row: 2 });
        assert_eq!(world.entity(id).unwrap().position(), Position::new(4, 2));
    }

    #[test]
    fn move_unknown_entity_is_error() {
        let (mut world, _id) = world_with_player();
        let err = world
            .try_move(EntityId(999), PointDelta::EAST)
            .unwrap_err();
        assert_eq!(err, WorldError::UnknownEntity(EntityId(999)));
    }
}
