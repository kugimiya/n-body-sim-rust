#[derive(Copy, Clone)]
pub struct Point(pub f64, pub f64);

impl Point {
    pub fn new(x: f64, y: f64) -> Point {
        Point(x, y)
    }

    pub fn length_square(&mut self) -> f64 {
        return self.0 * self.0 + self.1 * self.1;
    }

    pub fn length(&mut self) -> f64 {
        return f64::sqrt(self.length_square());
    }

    pub fn plus(&mut self, v2: Point) -> Point {
        return Point(self.0 + v2.0, self.1 + v2.1);
    }

    pub fn minus(&mut self, v2: Point) -> Point {
        return Point(self.0 - v2.0, self.1 - v2.1);
    }

    pub fn multiply(&mut self, v: f64) -> Point {
        return Point(self.0 * v, self.1 * v);
    }

    pub fn divide(&mut self, v: f64) -> Point {
        return Point(self.0 / v, self.1 / v);
    }

    pub fn as_tupl(&mut self) -> (f64, f64) {
        return (self.0, self.1);
    }
}
