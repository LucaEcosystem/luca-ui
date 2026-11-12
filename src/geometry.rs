//! Backend-independent geometry for layout and painting.
//!
//! All values are abstract layout units. This crate deliberately does not
//! define pixels, device scale factors, or platform surfaces: a visual
//! backend maps these units to its own coordinate system. Coordinates are
//! signed so children can be positioned relative to a parent origin; sizes
//! are unsigned.

/// A point in abstract layout units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// A width/height pair in abstract layout units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// An axis-aligned rectangle in abstract layout units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn from_origin_size(origin: Point, size: Size) -> Self {
        Self {
            x: origin.x,
            y: origin.y,
            width: size.width,
            height: size.height,
        }
    }

    /// Exclusive right edge.
    pub fn right(&self) -> i32 {
        self.x
            .saturating_add(self.width.min(i32::MAX as u32) as i32)
    }

    /// Exclusive bottom edge.
    pub fn bottom(&self) -> i32 {
        self.y
            .saturating_add(self.height.min(i32::MAX as u32) as i32)
    }

    pub fn origin(&self) -> Point {
        Point::new(self.x, self.y)
    }

    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    /// True when the point lies inside the half-open rectangle.
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x && point.x < self.right() && point.y >= self.y && point.y < self.bottom()
    }

    /// Shrink by `padding` on every side, clamping at zero size.
    pub fn inset(&self, padding: u32) -> Self {
        let doubled = padding.saturating_mul(2);
        Self {
            x: self.x.saturating_add(padding.min(i32::MAX as u32) as i32),
            y: self.y.saturating_add(padding.min(i32::MAX as u32) as i32),
            width: self.width.saturating_sub(doubled),
            height: self.height.saturating_sub(doubled),
        }
    }

    /// The overlapping area, or an empty rectangle when they do not overlap.
    pub fn intersect(&self, other: &Rect) -> Rect {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        if right <= x || bottom <= y {
            Rect::new(x, y, 0, 0)
        } else {
            Rect::new(x, y, (right - x) as u32, (bottom - y) as u32)
        }
    }
}
