//! A cell on the map, plus the mapping to screen/world coordinates.

use gametools::{Point, PointDelta};

/// A cell on the map, wrapping [`gametools::Point`] (`col`, `row`).
///
/// We wrap the library's point in our own type for two reasons:
///
/// 1. **Type safety** — a `Position` can't be confused with other `(i32, i32)`
///    pairs that happen to float around the code.
/// 2. **One place for conversions** — the screen mapping below lives here
///    instead of being scattered through the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position(Point);

impl Position {
    /// Creates a position from a column and a row. `(0, 0)` is the top-left cell.
    pub const fn new(col: i32, row: i32) -> Self {
        Self(Point::new(col, row))
    }

    /// The horizontal coordinate (column).
    pub const fn col(&self) -> i32 {
        self.0.col
    }

    /// The vertical coordinate (row).
    pub const fn row(&self) -> i32 {
        self.0.row
    }

    /// The underlying `gametools` point, for passing to grid helpers.
    pub const fn point(&self) -> Point {
        self.0
    }

    /// Moves this position by `delta`, returning the new cell. This does **not**
    /// check bounds or terrain — that is the [`World`](crate::world::World)'s job.
    pub fn offset(&self, delta: PointDelta) -> Self {
        Self(self.0 + delta)
    }

    /// Maps this cell to its **center** in 2D world space.
    ///
    /// The map's `row` grows downward, but most 2D coordinate systems (Bevy's
    /// included) grow `y` upward, so we negate the row. Cell `(0, 0)` therefore
    /// lands with its center at `(tile_size / 2, -tile_size / 2)`.
    ///
    /// Returns `(x, y)` in world units, not pixels; the caller decides the scale.
    pub fn to_world_xy(&self, tile_size: f32) -> (f32, f32) {
        let x = (self.0.col as f32 + 0.5) * tile_size;
        let y = -((self.0.row as f32) + 0.5) * tile_size;
        (x, y)
    }
}

impl From<Point> for Position {
    fn from(point: Point) -> Self {
        Self(point)
    }
}

impl From<Position> for Point {
    fn from(position: Position) -> Self {
        position.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gametools::PointDelta;

    #[test]
    fn world_xy_maps_col_to_x_and_row_to_negative_y() {
        // Cell (0,0), tile size 1: center at (0.5, -0.5).
        assert_eq!(Position::new(0, 0).to_world_xy(1.0), (0.5, -0.5));
        // Moving right (col up) raises x; moving down (row up) lowers y.
        assert_eq!(Position::new(3, 2).to_world_xy(1.0), (3.5, -2.5));
    }

    #[test]
    fn offset_applies_delta() {
        let moved = Position::new(2, 2).offset(PointDelta::NORTH);
        assert_eq!(moved, Position::new(2, 1));
    }
}
