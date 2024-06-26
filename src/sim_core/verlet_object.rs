use super::point::Point;
use rand::Rng;

#[derive(Copy, Clone)]
pub struct VerletObject {
    pub position: Point,
    pub position_last: Point,
    pub acceleration: Point,
    pub mass: f64,
    pub radius: f64,
    pub temp: f64,
    pub friction_factor: f64
}

impl VerletObject {
    pub fn new(
        x: f64,
        y: f64,
        mass: f64,
        radius: f64,
        initial_velocity_range_bounds: f64,
    ) -> VerletObject {
        let mut rnd = rand::thread_rng();

        VerletObject {
            position: Point::new(x, y),
            position_last: Point::new(
                x + rnd.gen_range(-1.0 * initial_velocity_range_bounds .. initial_velocity_range_bounds),
                y + rnd.gen_range(-1.0 * initial_velocity_range_bounds .. initial_velocity_range_bounds),
            ),
            acceleration: Point::new(0.0, 0.0),
            mass: mass,
            radius: radius,
            temp: 0.0,
            friction_factor: 0.0025,
        }
    }

    pub fn accelerate(&mut self, acceleration: Point) {
        self.acceleration = self.acceleration.plus(acceleration);
    }

    pub fn update(&mut self, dt: f64) {
        let mut velocity = self.position.minus(self.position_last.clone());
        self.position_last = self.position.clone();

        self.position = self
            .position
            .plus(velocity.plus(self.acceleration.multiply(dt * dt)));

        self.acceleration = Point::new(0.0, 0.0);
    }

    pub fn update_friction(&mut self) {
        let mut velocity = self.position.minus(self.position_last.clone());
        let velocity_length = f64::sqrt(velocity.length_square());
        self.position_last = self.position_last.plus(velocity.multiply(velocity_length * self.friction_factor));
    }

    pub fn temp_fix(&mut self) {
        if !self.temp.is_normal() {
            self.temp = 500_000.0;
        }

        self.temp -= self.temp * 0.00005;

        if self.temp < 0.0 {
            self.temp = 0.0;
        }

        if self.temp > 500_000.0 {
            self.temp = 500_000.0;
        }
    }
}
