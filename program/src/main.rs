pub mod modules;
use anyhow::Result;
use modules::{data::*, material::*};
use std::rc::Rc;

use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub fn main() -> Result<()> {
    pretty_env_logger::init();

    let data = CamData::default();

    //let image = image::open("program/resources/earth_4.jpg").unwrap();
    //let resources = Rc::new(Resources { earth: image });
    let _normal_material = Rc::new(NormalMaterial {});
    let _scene_info = SceneInfo::default();

    //let (mut canvas , mut event_pump) = get_canvas_and_pump(data.canvas_width as u32, data.canvas_height as u32);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Ray Tracer (Vulkan)")
        .with_inner_size(LogicalSize::new(
            data.canvas_width as u32,
            data.canvas_height as u32,
        ))
        .with_resizable(false)
        .build(&event_loop)?;

    let mut app = unsafe { modules::vulkan::App::create(&window)? };
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

            }.unwrap(),
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
