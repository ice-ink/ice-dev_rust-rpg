//! Terrain tiles and the two-dimensional map of tiles.
//!
//! The map is a rectangle of [`TileKind`] values. Storage and coordinate
//! arithmetic are handled by [`gametools::Grid`], which addresses cells with a
//! [`Point`] whose `col` is horizontal and `row` is vertical. Cell `(0, 0)` is
//! the top-left corner, `col` grows to the right and `row` grows downward.

use gametools::{GameResult, Grid, GridSize, GridTopology, Point, PointDelta};

/// The kind of terrain in a single cell.
///
/// There are only two kinds for now, but this enum is the natural place to
/// grow the world (water, doors, stairs, …) in later versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileKind {
    /// Walkable ground.
    Floor,
    /// A solid obstacle that blocks movement.
    Wall,
}

impl TileKind {
    /// Whether an entity is allowed to stand on this tile.
    pub fn is_passable(self) -> bool {
        matches!(self, TileKind::Floor)
    }
}

/// An error returned when a map operation references an invalid cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapError {
    /// The point lies outside the map's rectangle.
    OutOfBounds { col: i32, row: i32 },
}

impl std::fmt::Display for MapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapError::OutOfBounds { col, row } => {
                write!(f, "point ({col}, {row}) is outside the map")
            }
        }
    }
}

impl std::error::Error for MapError {}

/// A two-dimensional map of [`TileKind`] tiles.
pub struct Map {
    grid: Grid<TileKind>,
}

impl Map {
    /// Creates a `width × height` map where every cell is [`TileKind::Floor`].
    ///
    /// Returns a [`GameResult`] because the underlying grid validates that the
    /// dimensions are non-zero and small enough to fit in memory. `GameResult`
    /// is `gametools`' own `Result<T, GameError>`; we keep it here so a bad
    /// size is reported with the same rich error the grid itself would produce.
    pub fn new(width: usize, height: usize) -> GameResult<Self> {
        let grid = Grid::new(GridSize::new(width, height)?, TileKind::Floor)?;
        Ok(Self { grid })
    }

    /// Creates a `width × height` map whose border is [`TileKind::Wall`] and
    /// whose interior is [`TileKind::Floor`]. Handy for the movement demo.
    pub fn bordered(width: usize, height: usize) -> GameResult<Self> {
        let size = GridSize::new(width, height)?;
        let max_col = size.width() as i32 - 1;
        let max_row = size.height() as i32 - 1;
        let grid = Grid::new_with_fn(size, |point| {
            let is_border = point.col == 0
                || point.row == 0
                || point.col == max_col
                || point.row == max_row;
            if is_border {
                TileKind::Wall
            } else {
                TileKind::Floor
            }
        })?;
        Ok(Self { grid })
    }

    /// The map width, in cells (columns).
    pub fn width(&self) -> usize {
        self.grid.size().width()
    }

    /// The map height, in cells (rows).
    pub fn height(&self) -> usize {
        self.grid.size().height()
    }

    /// The tile at `point`, or `None` if `point` is out of bounds.
    pub fn tile_at(&self, point: Point) -> Option<TileKind> {
        self.grid.get(point).copied()
    }

    /// Whether `point` is inside the map's rectangle.
    pub fn is_in_bounds(&self, point: Point) -> bool {
        self.grid.is_in_bounds(point)
    }

    /// Whether an entity may stand on `point`: it is in bounds *and* its tile
    /// is passable.
    pub fn is_passable(&self, point: Point) -> bool {
        self.tile_at(point).is_some_and(|tile| tile.is_passable())
    }

    /// Sets the tile at `point`, returning an error if `point` is out of bounds.
    pub fn set_tile(&mut self, point: Point, kind: TileKind) -> Result<(), MapError> {
        if !self.is_in_bounds(point) {
            return Err(MapError::OutOfBounds {
                col: point.col,
                row: point.row,
            });
        }
        self.grid.set(point, kind);
        Ok(())
    }

    /// One bounded step from `origin` in direction `delta`.
    ///
    /// The map uses *bounded* topology: stepping off the edge yields `None`
    /// rather than wrapping around to the far side (which `gametools` also
    /// supports via [`GridTopology::Toroidal`]).
    pub fn step(&self, origin: Point, delta: PointDelta) -> Option<Point> {
        self.grid.step(origin, delta, GridTopology::Bounded)
    }

    /// Iterates over every in-bounds cell, row by row.
    pub fn points(&self) -> impl Iterator<Item = Point> + '_ {
        self.grid.points()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gametools::Point;

    #[test]
    fn new_map_is_all_floor() {
        let map = Map::new(3, 2).unwrap();
        for point in map.points() {
            assert_eq!(map.tile_at(point), Some(TileKind::Floor));
        }
    }

    #[test]
    fn bordered_map_has_wall_border_and_floor_interior() {
        let map = Map::bordered(4, 3).unwrap();
        // Corners and edges are walls.
        assert_eq!(map.tile_at(Point::new(0, 0)), Some(TileKind::Wall));
        assert_eq!(map.tile_at(Point::new(3, 2)), Some(TileKind::Wall));
        assert_eq!(map.tile_at(Point::new(2, 0)), Some(TileKind::Wall));
        // The interior is floor.
        assert_eq!(map.tile_at(Point::new(1, 1)), Some(TileKind::Floor));
        assert_eq!(map.tile_at(Point::new(2, 1)), Some(TileKind::Floor));
    }

    #[test]
    fn set_tile_rejects_out_of_bounds() {
        let mut map = Map::new(3, 3).unwrap();
        assert!(map.set_tile(Point::new(3, 0), TileKind::Wall).is_err());
        assert!(map.set_tile(Point::new(-1, 0), TileKind::Wall).is_err());
        assert!(map.set_tile(Point::new(1, 1), TileKind::Wall).is_ok());
        assert_eq!(map.tile_at(Point::new(1, 1)), Some(TileKind::Wall));
    }

    #[test]
    fn passability_requires_in_bounds() {
        let map = Map::new(2, 2).unwrap();
        assert!(map.is_passable(Point::new(0, 0)));
        // An out-of-bounds cell is never passable.
        assert!(!map.is_passable(Point::new(2, 0)));
    }
}
