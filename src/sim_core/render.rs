use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{WindowBuilder, Window},
};
use winit_input_helper::WinitInputHelper;
use tiny_skia::{Pixmap, Paint, Rect, Transform};

use super::verlet_world::VerletWorld;

pub struct Renderer {
    pub input: WinitInputHelper,
    pub window: Window,
    pub pixels: Pixels,
    pub drawing: Pixmap,
    pub width: u32,
    pub height: u32,
    pub draw_frames_in_output: bool
}

impl Renderer {
    pub fn new(width: u32, height: u32, event_loop: &mut EventLoop<()>, draw_frames_in_output: bool) -> Renderer {
        // let event_loop = EventLoop::new();
        let window = {
            let size = LogicalSize::new(width as f64, height as f64);
            let scaled_size = LogicalSize::new(width as f64, height as f64);
            WindowBuilder::new()
                .with_title("nbodysim-rust")
                .with_inner_size(scaled_size)
                .with_min_inner_size(size)
                .build(&event_loop)
                .unwrap()
        };
        let window_size = window.inner_size();

        return Renderer {
            input: WinitInputHelper::new(),
            drawing: Pixmap::new(width, height).unwrap(),
            pixels: Pixels::new(width, height, SurfaceTexture::new(window_size.width, window_size.height, &window)).unwrap(),
            window: window,
            width,
            height,
            draw_frames_in_output
        };
    }
}

pub fn draw(renderer: &mut Renderer, world: &mut VerletWorld) {
    let center_x = (renderer.width / 2) as f32;
    let center_y = (renderer.height / 2) as f32;

    // Fill all with black
    {
        let mut paint = Paint::default();
        paint.set_color_rgba8(0, 0, 0, 55);
        paint.anti_alias = false;

        let rect_result = Rect::from_xywh(0.0, 0.0, renderer.width as f32, renderer.height as f32).unwrap();
        renderer.drawing.fill_rect(rect_result, &paint, Transform::identity(), None);
    }

    // Draw chunks
    for chunk in world.chunks.iter() {
        let mut paint = Paint::default();
        paint.set_color_rgba8(0, 0, 255, 15);
        paint.anti_alias = false;

        let rect_result = Rect::from_xywh(center_x + (chunk.x * world.chunk_size) as f32, center_y + (chunk.y * world.chunk_size) as f32, world.chunk_size as f32, world.chunk_size as f32);
        if !rect_result.is_none() {
            renderer.drawing.fill_rect(rect_result.unwrap(), &paint, Transform::identity(), None);
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
            renderer.drawing.fill_rect(rect_result.unwrap(), &paint, Transform::identity(), None);
        } else {
            println!("ERROR: Rect creating failed, see next lines");
            println!("INFO: Object data: i={}, x={}, y={}, t={}, r={}", index, object.position.0, object.position.1, object.temp, object.radius);
            println!("INFO: Calculated to Rect: x={}, y={}, w={}, h={}", center_x + object.position.0 as f32, center_y + object.position.1 as f32, object.radius as f32, object.radius as f32);
            println!("INFO: Calculated in Rect: l={}, t={}, r={}, b={}", center_x + object.position.0 as f32, center_y + object.position.1 as f32, object.radius as f32 + center_x + object.position.0 as f32, object.radius as f32 + center_y + object.position.1 as f32);
            return;
        }

        index += 1;
    }

    // Save result to file
    if renderer.draw_frames_in_output {
        let mut fname = "output/image_".to_owned();
        fname.push_str(&format!("{:0>8}", world.step.to_string()));
        fname.push_str(".png");

        renderer.drawing.save_png(String::from(fname)).unwrap();
    }
}
