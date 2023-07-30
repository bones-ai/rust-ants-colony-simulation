#[derive(Clone, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub struct QuadTree {
    boundary: Rectangle,
    capacity: usize,
    points: Vec<Point>,

    tl: Option<Box<QuadTree>>,
    tr: Option<Box<QuadTree>>,
    bl: Option<Box<QuadTree>>,
    br: Option<Box<QuadTree>>,
}

impl QuadTree {
    pub fn new(boundary: Rectangle, capacity: usize) -> Self {
        Self {
            boundary,
            capacity,
            points: Vec::new(),
            tl: None,
            tr: None,
            bl: None,
            br: None,
        }
    }

    pub fn query(&self, range: &Rectangle) -> Vec<Point> {
        let mut res = Vec::new();

        if !self.boundary.intersects(&range) {
            return res;
        }

        for p in self.points.iter() {
            if range.contains(&p) {
                res.push(p.clone());
            }
        }

        if let Some(v) = &self.tl {
            res.extend(v.query(range));
        }
        if let Some(v) = &self.tr {
            res.extend(v.query(range));
        }
        if let Some(v) = &self.bl {
            res.extend(v.query(range));
        }
        if let Some(v) = &self.br {
            res.extend(v.query(range));
        }

        res
    }

    pub fn display(&self) {
        println!("vals: {:?}", self.points.clone());

        if let Some(v) = &self.tl {
            println!("tl");
            v.display();
        }
        if let Some(v) = &self.tr {
            println!("tr");
            v.display();
        }
        if let Some(v) = &self.bl {
            println!("bl");
            v.display();
        }
        if let Some(v) = &self.br {
            println!("br");
            v.display();
        }
    }

    pub fn subdivide(&mut self) {
        if self.tl.is_some() {
            return;
        }

        let x = self.boundary.x;
        let y = self.boundary.y;
        let w = self.boundary.w;
        let h = self.boundary.h;

        let tl_rect = Rectangle::new(x - w / 2.0, y - h / 2.0, w / 2.0, h / 2.0);
        self.tl = Some(Box::new(QuadTree::new(tl_rect, self.capacity)));
        let tr_rect = Rectangle::new(x + w / 2.0, y - h / 2.0, w / 2.0, h / 2.0);
        self.tr = Some(Box::new(QuadTree::new(tr_rect, self.capacity)));
        let bl_rect = Rectangle::new(x + w / 2.0, y + h / 2.0, w / 2.0, h / 2.0);
        self.bl = Some(Box::new(QuadTree::new(bl_rect, self.capacity)));
        let br_rect = Rectangle::new(x - w / 2.0, y + h / 2.0, w / 2.0, h / 2.0);
        self.br = Some(Box::new(QuadTree::new(br_rect, self.capacity)));
    }

    pub fn insert(&mut self, point: &Point) -> bool {
        if !self.boundary.contains(point) {
            return false;
        }

        if self.points.len() < self.capacity {
            self.points.push(point.clone());
            return true;
        }

        self.subdivide();

        if let Some(v) = &mut self.tl {
            if v.insert(&point) {
                return true;
            }
        }
        if let Some(v) = &mut self.tr {
            if v.insert(&point) {
                return true;
            }
        }
        if let Some(v) = &mut self.bl {
            if v.insert(&point) {
                return true;
            }
        }
        if let Some(v) = &mut self.br {
            if v.insert(&point) {
                return true;
            }
        }

        return false;
    }
}

impl Rectangle {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(&self, other: &Point) -> bool {
        other.x >= self.x - self.w
            && other.x < self.x + self.w
            && other.y >= self.y - self.h
            && other.y < self.y + self.h
    }

    pub fn intersects(&self, other: &Rectangle) -> bool {
        !(other.x - other.w > self.x + self.w
            || other.x + other.w < self.x - self.w
            || other.y - other.h > self.y + self.h
            || other.y + other.h < self.y - self.h)
    }
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
