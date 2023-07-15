pub mod modules;
use rand::prelude::*;
use modules::{data::*, material::*, trace::*};
use std::rc::Rc;
use vector3d::Vector3d;
use crate::modules::get_canvas_and_pump;
use crate::modules::RayReturnState::Ray;
use anyhow::Result;

use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};


const SHADER: &[u8] = include_bytes!(env!("shader.spv"));



pub fn main() -> Result<()> {
    let data = CamData::default();

    let image = image::open("src/resources/earth_4.jpg").unwrap();
    let resources = Rc::new(Resources { earth: image });
    let _normal_material = Rc::new(NormalMaterial {});
    let scene_info = SceneInfo::default();

    //let (mut canvas , mut event_pump) = get_canvas_and_pump(data.canvas_width as u32, data.canvas_height as u32);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vulkan Tutorial (Rust)")
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)?;

    let mut destroying = false;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            // Render a frame if our Vulkan app is not being destroyed.
            Event::MainEventsCleared if !destroying => {},
                //unsafe { app.render(&window) }.unwrap(),
                //render here
            // Destroy our Vulkan app.
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                destroying = true;
                *control_flow = ControlFlow::Exit;
                //unsafe { app.destroy(); }
            }
            _ => {}
        }
    });


    println!("rendering...");

    /*if modules::Ray::render(&mut canvas, &mut event_pump, &data, &scene_info, resources) {
        return;
    }*/











    println!("done rendering");

    Ok(())

    /*canvas.present();

    loop {
        for event in event_pump.poll_iter() {
            if let sdl2::event::Event::Quit { .. } = event {
                return;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100))
    }*/
}
