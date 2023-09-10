pub mod modules;
use anyhow::Result;
use std::rc::Rc;

use shared::*;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

pub fn main() -> Result<()> {
    pretty_env_logger::init();


    let cam_data = CamData {
        pos: glam::Vec4::new(0.0, 0.0, 0.0, 0.0),
        orientation: glam::Vec4::new(0.0, 0.0, 1.0, 0.0),
        canvas_width: WIDTH as u32,
        canvas_height: HEIGHT as u32,
        fov: 90.0,
        samples: 1,
    };

    //let image = image::open("program/resources/earth_4.jpg").unwrap();
    //let resources = Rc::new(Resources { earth: image });
    //let normal_material = Rc::new(NormalMaterial {});
    let scene_info = SceneInfo {
        sun_orientation: Vector3d::new(1.0, -1.0, 1.0),
        hittable_objects: [
            Sphere::new(Vector3d::new(0.0, 0.0, 2.0), 0.5),
            Sphere::new(Vector3d::new(1.0, -0.2, 2.0), 0.3),
            Sphere::new(Vector3d::new(-1.0, -0.2, 2.0), 0.3),
        ],
    };

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Ray Tracer (Vulkan)")
        .with_inner_size(LogicalSize::new(
            cam_data.canvas_width,
            cam_data.canvas_height,
        ))
        .with_resizable(false)
        .build(&event_loop)?;

    let mut app = unsafe { modules::vulkan::App::create(&window, cam_data, scene_info)? };
    let mut destroying = false;
    let mut frame_count = 0;
    let mut start_time = std::time::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            // Render a frame if our Vulkan app is not being destroyed.
            Event::MainEventsCleared if !destroying => unsafe {
                frame_count += 1;
                let elapsed = start_time.elapsed().as_secs_f32();
                if elapsed > 1.0 {
                    let fps = frame_count as f32 / elapsed;
                    println!("FPS: {}", fps);
                    frame_count = 0;
                    start_time = std::time::Instant::now();
                }
                app.render(&window)
            }
            .unwrap(),
            // Destroy our Vulkan app.
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                destroying = true;
                *control_flow = ControlFlow::Exit;
                unsafe {
                    app.destroy();
                }
            }
            _ => {}
        }
    });
}
