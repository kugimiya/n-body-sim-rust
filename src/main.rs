#![feature(get_many_mut)]

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

use rand::Rng;
use std::time::{Duration, Instant};

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend
}

impl App {
    fn render(&mut self, args: &RenderArgs, objects: &mut Vec<VerletObject>) {
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        self.gl.draw(args.viewport(), |c, gl| {
            clear(BLACK, gl);

            let transform = c
                .transform
                .trans(0.0, 0.0);

            for object in objects {
                let square = rectangle::square(object.position.0, object.position.1, 1.0);
                rectangle(RED, square, transform, gl);
            }
        });
    }

    fn update(&mut self, _args: &UpdateArgs) {
        // idk, useless method now  :^)
    }
}

struct VerletObject {
    position: Point,
    position_last: Point,
    acceleration: Point,
    mass: f64,
    radius: f64,
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("n-body-square", [860, 860])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl)
    };

    let mut rnd = rand::thread_rng();
    let mut objects: Vec<VerletObject> = Vec::new();

    let base_dt = 0.005;
    let sub_steps = 8;
    let objects_count = 1000;
    let mut step = 0;

    for _step in 0..objects_count {
        let position = (rnd.gen_range(110.0..750.0), rnd.gen_range(110.0..750.0));

        objects.push(VerletObject {
            position: Point(position.0, position.1),
            position_last: Point(position.0, position.1),
            acceleration: Point(0.0, 0.0),
            mass: 40.0,
            radius: 0.5,
        });
    }

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args, &mut objects);
        }

        if let Some(args) = e.update_args() {
            step += 1;
            println!("INFO: Run step №{}", step);
            resolve_gravity(&mut objects);

            for _step in 0..sub_steps {
                resolve_collisions(&mut objects);
                update_objects(&mut objects, base_dt / f64::from(sub_steps));
            }

            app.update(&args);
        }
    }
}

fn resolve_collisions(objects: &mut Vec<VerletObject>) {
    let start = Instant::now();
    let mut iterations: i64 = 0;

    for i in 0..objects.len() {
        for j in i..objects.len() {
            if i == j {
                continue;
            }

            let [object1, object2] = objects.get_many_mut([i, j]).unwrap();

            if apply_collisions(object1, object2) {
                iterations += 1;
            }
        }
    }

    let duration: Duration = start.elapsed();
    println!(
        "INFO: Collisions resolved: {} for {:?}",
        iterations, duration
    );
}

fn resolve_gravity(objects: &mut Vec<VerletObject>) {
    let start = Instant::now();
    let mut iterations: i64 = 0;
    let gravity = 6.674;

    for i in 0..objects.len() {
        for j in i..objects.len() {
            if i == j {
                continue;
            }

            iterations += 1;

            let [object1, object2] = objects.get_many_mut([i, j]).unwrap();

            let mut velocity = object1.position.minus(Point(object2.position.0, object2.position.1));
            let velocity_squared = velocity.length_square();
            let force = gravity * ((object1.mass * object2.mass) / velocity_squared);
            let acceleration = force / f64::sqrt(velocity_squared);

            accelerate_object(
                object1,
                object2
                    .position
                    .minus(Point(object1.position.0, object1.position.1))
                    .multiply(acceleration),
            );

            accelerate_object(
                object2,
                object1
                    .position
                    .minus(Point(object2.position.0, object2.position.1))
                    .multiply(acceleration),
            );
        }
    }
    let duration: Duration = start.elapsed();
    println!(
        "INFO: Iterations for gravity: {} for {:?}",
        iterations, duration
    );
}

fn update_objects(objects: &mut Vec<VerletObject>, dt: f64) {
    for object1 in objects.iter_mut() {
        update_object(object1, dt);
    }
}

fn apply_collisions(object1: &mut VerletObject, object2: &mut VerletObject) -> bool {
    let collide_responsibility = 0.375;
    let mut velocity = object1
        .position
        .minus(Point(object2.position.0, object2.position.1));
    let distance_squared = velocity.length_square();
    let distance_minimal = object1.radius + object2.radius;

    // no overlap, skip
    if distance_squared > (distance_minimal * distance_minimal) {
        return false;
    }

    let distance = f64::sqrt(distance_squared);
    let mut diff = velocity.divide(distance);

    let common_mass = object1.mass + object2.mass;
    let object1_mass_ratio = object1.mass / common_mass;
    let object2_mass_ratio = object2.mass / common_mass;

    let delta = collide_responsibility * (distance - distance_minimal);

    object1.position = object1
        .position
        .minus(diff.multiply(object2_mass_ratio * delta).divide(2.0));

    object2.position = object2
        .position
        .plus(diff.multiply(object1_mass_ratio * delta).divide(2.0));

    return true;
}

fn accelerate_object(object: &mut VerletObject, acceleration: Point) {
    object.acceleration = object.acceleration.plus(acceleration);
}

fn update_object(object: &mut VerletObject, dt: f64) {
    let mut velocity = object
        .position
        .minus(Point(object.position_last.0, object.position_last.1));
    object.position_last = Point(object.position.0, object.position.1);

    object.position = object
        .position
        .plus(velocity.plus(object.acceleration.multiply(dt * dt)));

    object.acceleration = Point(0.0, 0.0);
}

struct Point(f64, f64);

impl Point {
    fn length_square(&mut self) -> f64 {
        return self.0 * self.0 + self.1 * self.1;
    }

    fn plus(&mut self, v2: Point) -> Point {
        return Point(self.0 + v2.0, self.1 + v2.1);
    }

    fn minus(&mut self, v2: Point) -> Point {
        return Point(self.0 - v2.0, self.1 - v2.1);
    }

    fn multiply(&mut self, v: f64) -> Point {
        return Point(self.0 * v, self.1 * v);
    }

    fn divide(&mut self, v: f64) -> Point {
        return Point(self.0 / v, self.1 / v);
    }
}
