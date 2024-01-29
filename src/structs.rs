use windows::Win32::Foundation::RECT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rectangle {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Rectangle {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_points(location: Point, size: Size) -> Rectangle {
        Rectangle {
            x: location.x,
            y: location.y,
            width: size.width,
            height: size.height,
        }
    }

    pub fn from_ltrb(left: i32, top: i32, right: i32, bottom: i32) -> Rectangle {
        Rectangle {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }
    }

    pub fn location(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    pub fn size(&self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    pub fn left(&self) -> i32 {
        self.x
    }

    pub fn top(&self) -> i32 {
        self.y
    }

    pub fn right(&self) -> i32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.height
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0 && self.height == 0 && self.x == 0 && self.y == 0
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    pub fn contains_point(&self, point: Point) -> bool {
        self.contains(point.x, point.y)
    }

    pub fn contains_rect(&self, rect: Rectangle) -> bool {
        self.contains(rect.x, rect.y) && self.contains(rect.right(), rect.bottom())
    }

    pub fn inflate(&mut self, width: i32, height: i32) {
        self.x -= width;
        self.y -= height;
        self.width += 2 * width;
        self.height += 2 * height;
    }

    pub fn intersect(&mut self, other: Rectangle) {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = self.right().min(other.right());
        let y2 = self.bottom().min(other.bottom());

        if x2 >= x1 && y2 >= y1 {
            *self = Rectangle::new(x1, y1, x2 - x1, y2 - y1);
        } else {
            *self = Rectangle::new(0, 0, 0, 0); // Empty Rectangle
        }
    }

    pub fn intersect_with(&self, other: Rectangle) -> Rectangle {
        let mut result = *self;
        result.intersect(other);
        result
    }

    pub fn intersects_with(&self, other: Rectangle) -> bool {
        self.x < other.x + other.width
            && other.x < self.x + self.width
            && self.y < other.y + other.height
            && other.y < self.y + self.height
    }

    pub fn union_with(&self, other: Rectangle) -> Rectangle {
        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = self.right().max(other.right());
        let y2 = self.bottom().max(other.bottom());

        Rectangle::new(x1, y1, x2 - x1, y2 - y1)
    }

    pub fn offset(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
    }
}

impl From<RECT> for Rectangle {
    fn from(rect: RECT) -> Self {
        Self {
            x: rect.left,
            y: rect.top,
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
        }
    }
}
