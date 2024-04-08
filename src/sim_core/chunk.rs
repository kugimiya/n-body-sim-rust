use super::point::Point;

pub struct Chunk {
    pub x: i32,
    pub y: i32,
    pub indecies: Vec<i32>,

    pub mass_center: Point,
    pub mass: f64,
}
