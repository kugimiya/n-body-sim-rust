#![deny(clippy::all)]
#![forbid(unsafe_code)]

#![feature(get_many_mut)]

use error_iter::ErrorIter as _;
use log::{debug, error};
use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event,VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;
use tiny_skia::{Pixmap, Paint, Rect, Transform};

use rand::Rng;
use std::time::{Duration, Instant};

struct VerletObject {
    position: Point,
    position_last: Point,
    acceleration: Point,
    mass: f64,
    radius: f64,
    temp: f64,
}

struct Chunk {
    x: i32,
    y: i32,
    indecies: Vec<i32>,
}

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scaled_size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("nbodysim-rust")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut drawing = Pixmap::new(WIDTH, HEIGHT).unwrap();

    let mut rnd = rand::thread_rng();
    let mut objects: Vec<VerletObject> = Vec::new();
    let mut chunks: Vec<Chunk> = Vec::new();

    let base_dt = 0.005;
    let sub_steps = 8;
    let objects_count = 64000;
    let mut step = 0;
    let mut chunk_size = 32;
    let mut last_collision_resolve_duration = 0.0;

    // Create objects
    for _step in 0..objects_count {
        let position = (rnd.gen_range(-1000.0..1000.0), rnd.gen_range(-600.0..600.0));

        objects.push(VerletObject {
            position: Point(position.0, position.1),
            position_last: Point(position.0 - rnd.gen_range(-0.25..0.25), position.1 - rnd.gen_range(-0.25..0.25)),
            acceleration: Point(0.0, 0.0),
            mass: rnd.gen_range(10.0..90.0),
            radius: rnd.gen_range(0.1..0.5),
            temp: 0.0,
        });
    }

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            pixels.frame_mut().copy_from_slice(drawing.data());
            if let Err(err) = pixels.render() {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Update world
            step += 1;
            println!("INFO: Run step №{} with ch_size {}", step, chunk_size);

            let mut cur_collision_resolve_duration = 0.0;

            for _step in 0..sub_steps {
                let duration = resolve_collisions_opti(&mut objects, &mut chunks, base_dt / f64::from(sub_steps));

                cur_collision_resolve_duration = (cur_collision_resolve_duration + duration) / 2.0;
                update_objects(&mut objects, base_dt / f64::from(sub_steps), chunk_size, &mut chunks);
            }

            if ((last_collision_resolve_duration + cur_collision_resolve_duration) / 2.0) < cur_collision_resolve_duration {
                chunk_size = chunk_size + 2;
            } else {
                chunk_size = chunk_size - 2;
            }

            if chunk_size < 4 {
                chunk_size = 4;
            }

            if chunk_size > 48 {
                chunk_size = 48;
            }

            last_collision_resolve_duration = cur_collision_resolve_duration;

            resolve_gravity(&mut objects, base_dt);
            update_objects(&mut objects, base_dt / f64::from(sub_steps), chunk_size, &mut chunks);

            // Fill all with black
            {
                let mut paint = Paint::default();
                paint.set_color_rgba8(0, 0, 0, 55);
                paint.anti_alias = false;

                let rect_result = Rect::from_xywh(0.0, 0.0, WIDTH as f32, HEIGHT as f32);
                if !rect_result.is_none() {
                    drawing.fill_rect(rect_result.unwrap(), &paint, Transform::identity(), None);
                }
            }

            let center_x = (WIDTH / 2) as f32;
            let center_y = (HEIGHT / 2) as f32;

            // Draw chunks
            for chunk in chunks.iter() {
                let mut paint = Paint::default();
                paint.set_color_rgba8(0, 0, 255, 15);
                paint.anti_alias = false;

                let rect_result = Rect::from_xywh(center_x + (chunk.x * chunk_size) as f32, center_y + (chunk.y * chunk_size) as f32, chunk_size as f32, chunk_size as f32);
                if !rect_result.is_none() {
                    drawing.fill_rect(rect_result.unwrap(), &paint, Transform::identity(), None);
                }
            }

            // Draw objects
            let mut index = 0;
            for object in objects.iter_mut() {
                let mut paint = Paint::default();
                paint.set_color_rgba8(object.temp as u8, 255 - object.temp as u8, object.temp as u8, 255);
                paint.anti_alias = false;

                let rect_result = Rect::from_xywh(center_x + object.position.0 as f32, center_y + object.position.1 as f32, object.radius as f32, object.radius as f32);

                if !rect_result.is_none() {
                    drawing.fill_rect(rect_result.unwrap(), &paint, Transform::identity(), None);
                } else {
                    println!("ERROR: Rect creating failed, see next lines");
                    println!("INFO: Object data: i={}, x={}, y={}, t={}, r={}", index, object.position.0, object.position.1, object.temp, object.radius);
                    println!("INFO: Calculated to Rect: x={}, y={}, w={}, h={}", center_x + object.position.0 as f32, center_y + object.position.1 as f32, object.radius as f32, object.radius as f32);
                    println!("INFO: Calculated in Rect: l={}, t={}, r={}, b={}", center_x + object.position.0 as f32, center_y + object.position.1 as f32, object.radius as f32 + center_x + object.position.0 as f32, object.radius as f32 + center_y + object.position.1 as f32);

                    *control_flow = ControlFlow::Exit;
                    return;
                }

                index += 1;
            }

            // Save result to file
            let mut fname = "output/image_".to_owned();
            fname.push_str(&format!("{:0>8}", step.to_string()));
            fname.push_str(".png");

            drawing.save_png(String::from(fname)).unwrap();

            // Rerender
            window.request_redraw();
        }
    });
}

fn resolve_collisions_opti(objects: &mut Vec<VerletObject>, chunks: &mut Vec<Chunk>, dt: f64) -> f64 {
    let start = Instant::now();
    let mut iterations: i64 = 0;

    for chunk_index in 0..chunks.len() {
        let chunk = chunks.get(chunk_index).unwrap();
        let hashes: [(i32, i32); 5] = [
            (chunk.x, chunk.y),
            (chunk.x - 1, chunk.y),
            (chunk.x + 1, chunk.y),
            (chunk.x, chunk.y - 1),
            (chunk.x, chunk.y + 1),
        ];

        let mut object_indecies: Vec<i32> = Vec::new();

        for chunk_hash in hashes {
            let search_result = chunks.iter().find(|ch| ch.x == chunk_hash.0 && ch.y == chunk_hash.1);
            if !search_result.is_none() {
                let chunk = search_result.unwrap();
                for i in chunk.indecies.iter() {
                    if !object_indecies.contains(i) || object_indecies.len() == 0 {
                        object_indecies.push(i + 0);
                    }
                }
            }
        }

        for i in 0..object_indecies.len() {
            for j in i..object_indecies.len() {
                if i == j {
                    continue;
                }

                let get_result = objects.get_many_mut([object_indecies[i] as usize, object_indecies[j] as usize]);

                if get_result.is_err() {
                    continue;
                } else {
                    let [object1, object2] = get_result.unwrap();
                    if apply_collisions(object1, object2, dt) {
                        iterations += 1;
                    }
                }
            }
        }
    }

    let duration: Duration = start.elapsed();
    // println!(
    //     "INFO: Collisions resolved: {} for {:?}",
    //     iterations, duration
    // );

    return duration.as_millis() as f64;
}

fn resolve_collisions_bruteforce(objects: &mut Vec<VerletObject>, dt: f64) {
    let start = Instant::now();
    let mut iterations: i64 = 0;

    for i in 0..objects.len() {
        for j in i..objects.len() {
            if i == j {
                continue;
            }

            let [object1, object2] = objects.get_many_mut([i, j]).unwrap();

            if apply_collisions(object1, object2, dt) {
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

fn resolve_gravity(objects: &mut Vec<VerletObject>, _dt: f64) {
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
    // println!(
    //     "INFO: Iterations for gravity: {} for {:?}",
    //     iterations, duration
    // );
}

fn update_objects(objects: &mut Vec<VerletObject>, dt: f64, chunk_size: i32, chunks: &mut Vec<Chunk>) {
    chunks.clear();

    for object_index in 0..objects.len() {
        let object1 = objects.get_mut(object_index).unwrap();
        update_object(object1, dt);
        push_to_chunks(object1, object_index, chunk_size, chunks);
    }
}

fn push_to_chunks(object1: &mut VerletObject, object_index: usize, chunk_size: i32, chunks: &mut Vec<Chunk>) {
    let (chunk_x, chunk_y) = position_to_chunk_coord(object1, chunk_size);
    let chunk_position_in_vec = chunks.iter().position(|ch| ch.x == chunk_x && ch.y == chunk_y);
    if chunk_position_in_vec.is_none() {
        // create
        let mut indecies: Vec<i32> = Vec::new();
        indecies.push(object_index as i32);

        chunks.push(Chunk {
            x: chunk_x,
            y: chunk_y,
            indecies: indecies
        });
    } else {
        // andrew mutate
        let chunk_pos = chunk_position_in_vec.unwrap();
        let chunk = chunks.get_mut(chunk_pos).unwrap();
        chunk.indecies.push(object_index as i32);
    }
}

fn position_to_chunk_coord(object: &mut VerletObject, chunk_size: i32) -> (i32, i32) {
    return (
        f64::floor(object.position.0 / f64::from(chunk_size)) as i32, 
        f64::floor(object.position.1 / f64::from(chunk_size)) as i32
    );
}

fn apply_collisions(object1: &mut VerletObject, object2: &mut VerletObject, _dt: f64) -> bool {
    let collide_responsibility = 0.375;
    let mut velocity = object1
        .position
        .minus(Point(object2.position.0, object2.position.1));
    let distance_squared = velocity.length_square();
    let distance_minimal = object1.radius + object2.radius;

    // no overlap, skip
    if distance_squared >= (distance_minimal * distance_minimal) {
        return false;
    }

    let distance = f64::sqrt(distance_squared);
    let mut diff = velocity.divide(distance);

    let common_mass = object1.mass + object2.mass;
    let object1_mass_ratio = object1.mass / common_mass;
    let object2_mass_ratio = object2.mass / common_mass;

    let delta = collide_responsibility * (distance - distance_minimal);

    object1.position = object1.position.minus(diff.multiply(object2_mass_ratio * delta).divide(2.0));
    object2.position = object2.position.plus(diff.multiply(object1_mass_ratio * delta).divide(2.0));

    let object1_speed = object1.position.minus(Point(object1.position_last.0, object1.position_last.1)).length_square();
    let object2_speed = object2.position.minus(Point(object2.position_last.0, object2.position_last.1)).length_square();

    object1.temp += common_mass * object2_speed * object2_speed * 25.0;
    object2.temp += common_mass * object1_speed * object1_speed * 25.0;

    let temp_to_obj1 = object2.temp * (object2_mass_ratio * 0.075);
    let temp_to_obj2 = object1.temp * (object1_mass_ratio * 0.075);

    object1.temp = object1.temp + temp_to_obj1 - temp_to_obj2;
    object2.temp = object2.temp + temp_to_obj2 - temp_to_obj1;

    return true;
}

fn accelerate_object(object: &mut VerletObject, acceleration: Point) {
    object.acceleration = object.acceleration.plus(acceleration);
}

fn update_object(object: &mut VerletObject, dt: f64) {
    let mut velocity = object.position.minus(Point(object.position_last.0, object.position_last.1));
    object.position_last = Point(object.position.0, object.position.1);

    object.position = object
        .position
        .plus(velocity.plus(object.acceleration.multiply(dt * dt)));

    // implementation of friction :^)
    velocity = object.position.minus(Point(object.position_last.0, object.position_last.1));
    let velocity_length = f64::sqrt(velocity.length_square());
    let friction_factor = 0.0085;
    object.position_last = object.position_last.plus(velocity.multiply(velocity_length * friction_factor));

    object.temp -= object.temp * 0.00005;
    if object.temp < 0.0 {
        object.temp = 0.0;
    }
    if object.temp > 1_000_000.0 {
        object.temp = 1_000.0 ;
    }

    object.acceleration = Point(0.0, 0.0);

    // correct object position if it went crazy
    if (object.position.0 as f32 == f32::INFINITY) || (object.position.1 as f32 == f32::INFINITY) || object.position.0.is_nan() || object.position.0.is_nan() {
        object.position.0 = 0.0;
        object.position.1 = 0.0;
        object.position_last.0 = 0.0;
        object.position_last.1 = 0.0;
    }
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
