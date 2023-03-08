/// A rectangle, defined by it's upper-left and upper-right corners
pub struct Rect {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

impl Rect {
    /// Create a new rectangle with its top-left corner located at (`x`, `y`),
    /// and with a specified width and height.
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x1: x,
            y1: y,
            x2: x + width,
            y2: y + height,
        }
    }

    /// Returns true if this rectangle overlaps another rectangle.
    pub const fn intersect(&self, other: &Rect) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }

    /// Returns a coordinate, rounded down (floor) to the nearest integer,
    /// describing the center of this rectangle.
    pub const fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }
}

/// Create a new rectangle from a pair of coordinates (x1, y1) and (x2, y2)
/// locating the upper-left and bottom-right corners, respectively.
impl From<((i32, i32), (i32, i32))> for Rect {
    fn from(((x1, y1), (x2, y2)): ((i32, i32), (i32, i32))) -> Self {
        Self { x1, y1, x2, y2 }
    }
}
