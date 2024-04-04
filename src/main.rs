#![deny(clippy::all)]
#![forbid(unsafe_code)]

#![feature(get_many_mut)]

mod sim_core;
use sim_core::verlet_world::verlet_world::VerletWorld;

use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event,VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;
use tiny_skia::{Pixmap, Paint, Rect, Transform};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

const WIDTH_RAND: f64 = 200.0;
const HEIGHT_RAND: f64 = 200.0;

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

    let mut world = VerletWorld::new(1024);
    world.fill(WIDTH_RAND, HEIGHT_RAND, 0.25, 10.0..20.0, 1.0..2.0);

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
            world.update();

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
            for chunk in world.chunks.iter() {
                let mut paint = Paint::default();
                paint.set_color_rgba8(0, 0, 255, 15);
                paint.anti_alias = false;

                let rect_result = Rect::from_xywh(center_x + (chunk.x * world.chunk_size) as f32, center_y + (chunk.y * world.chunk_size) as f32, world.chunk_size as f32, world.chunk_size as f32);
                if !rect_result.is_none() {
                    drawing.fill_rect(rect_result.unwrap(), &paint, Transform::identity(), None);
                }
            }

            // Draw objects
            let mut index = 0;
            for object in world.objects.iter_mut() {
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
            fname.push_str(&format!("{:0>8}", world.step.to_string()));
            fname.push_str(".png");

            drawing.save_png(String::from(fname)).unwrap();

            // Rerender
            window.request_redraw();
        }
    });
}
