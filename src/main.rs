#![deny(clippy::all)]
#![forbid(unsafe_code)]

#![feature(get_many_mut)]

mod sim_core;
use sim_core::verlet_world::VerletWorld;
use sim_core::render::{Renderer, draw};
use winit::{
    event::{Event,VirtualKeyCode},
    event_loop::{EventLoop,ControlFlow},
};

const CANVAS_WIDTH: u32 = 1920;
const CANVAS_HEIGHT: u32 = 1080;

const OBJECTS_COUNT: i32 = 2000;
const MAX_OBJECTS_COUNT: i32 = 1000;
const WORLD_RADIUS: f64 = 1080.0 / 2.0;
const SPAWN_WIDTH_BOUND: f64 = 1080.0 / 2.0; // from -x to x
const SPAWN_HEIGHT_BOUND: f64 = 10.1; // from -y to y
const OBJECT_INIT_VELOCITY_BOUND: f64 = 0.1; // from -v to v
const OBJECT_MASS_RANGE: std::ops::Range<f64> = 1.0..50.0;
const OBJECT_RADIUS_RANGE: std::ops::Range<f64> = 0.1..2.0;
const DRAW_OUTPUT: bool = false;
const CIRCLED_FILL: bool = false;

fn main() {
    let mut event_loop = EventLoop::new();
    let mut world = VerletWorld::new(OBJECTS_COUNT, WORLD_RADIUS, MAX_OBJECTS_COUNT);
    let mut renderer = Renderer::new(CANVAS_WIDTH, CANVAS_HEIGHT, &mut event_loop, DRAW_OUTPUT);

    world.fill(SPAWN_WIDTH_BOUND, SPAWN_HEIGHT_BOUND, OBJECT_INIT_VELOCITY_BOUND, OBJECT_MASS_RANGE, OBJECT_RADIUS_RANGE, CIRCLED_FILL);
    event_loop.run(move |event, _, control_flow| {
        // Loop iteration
        if let Event::RedrawRequested(_) = event {
            renderer.pixels.frame_mut().copy_from_slice(renderer.drawing.data());
            if let Err(err) = renderer.pixels.render() {
                println!("ERROR: {:?}", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Update event
        if renderer.input.update(&event) {
            // Close events
            if renderer.input.key_pressed(VirtualKeyCode::Escape) || renderer.input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Update world
            world.update();

            // FIXME: for perf measurement task :^)
            world.fill(SPAWN_WIDTH_BOUND, SPAWN_HEIGHT_BOUND, OBJECT_INIT_VELOCITY_BOUND, OBJECT_MASS_RANGE, OBJECT_RADIUS_RANGE, CIRCLED_FILL);

            // Draw
            draw(&mut renderer, &mut world);

            // Re-render
            renderer.window.request_redraw();
        }
    }); 
}
