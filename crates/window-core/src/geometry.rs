//! Primitive geometry in GUI-space pixels.
//!
//! GUI space: integer pixels, origin at the top-left of the container GUI
//! texture, y growing downward.

/// A point in GUI space.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Point {
    /// Horizontal position in pixels.
    pub x: i32,
    /// Vertical position in pixels.
    pub y: i32,
}

impl Point {
    /// Construct a point.
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// A size in pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Size {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl Size {
    /// Construct a size.
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// An axis-aligned rectangle in GUI space.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rect {
    /// Left edge.
    pub x: i32,
    /// Top edge.
    pub y: i32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl Rect {
    /// Construct a rectangle.
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    /// The 18x18 vanilla slot box around a 16x16 slot item rect.
    pub(crate) const fn slot_box(&self) -> Self {
        Self::new(self.x - 1, self.y - 1, self.width + 2, self.height + 2)
    }

    /// Construct from a position and size.
    pub const fn from_parts(pos: Point, size: Size) -> Self {
        Self::new(pos.x, pos.y, size.width, size.height)
    }

    /// The exclusive right edge (`x + width`).
    pub const fn right(&self) -> i32 {
        self.x + self.width as i32
    }

    /// The exclusive bottom edge (`y + height`).
    pub const fn bottom(&self) -> i32 {
        self.y + self.height as i32
    }

    /// Top-left corner.
    pub const fn origin(&self) -> Point {
        Point::new(self.x, self.y)
    }

    /// Size of the rectangle.
    pub const fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Whether this rectangle and `other` overlap (touching edges do not count).
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.right() && other.x < self.right() && self.y < other.bottom() && other.y < self.bottom()
    }

    /// Whether `other` lies entirely within `self`.
    pub fn contains(&self, other: &Rect) -> bool {
        other.x >= self.x && other.y >= self.y && other.right() <= self.right() && other.bottom() <= self.bottom()
    }

    /// Smallest rectangle covering both `self` and `other`.
    pub fn union(&self, other: &Rect) -> Rect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Rect::new(x, y, (right - x) as u32, (bottom - y) as u32)
    }
}

/// Per-edge insets (used by 9-slice frames and padding).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Insets {
    /// Top inset in pixels.
    pub top: u32,
    /// Right inset in pixels.
    pub right: u32,
    /// Bottom inset in pixels.
    pub bottom: u32,
    /// Left inset in pixels.
    pub left: u32,
}

impl Insets {
    /// Equal insets on all four edges.
    pub const fn uniform(v: u32) -> Self {
        Self { top: v, right: v, bottom: v, left: v }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_edges_and_union() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 5, 10, 10);
        assert_eq!(a.right(), 10);
        assert_eq!(b.bottom(), 15);
        assert!(a.intersects(&b));
        assert_eq!(a.union(&b), Rect::new(0, 0, 15, 15));
    }

    #[test]
    fn touching_rects_do_not_intersect() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(10, 0, 10, 10);
        assert!(!a.intersects(&b));
    }
}
