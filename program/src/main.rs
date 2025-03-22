pub mod modules;
use glam::Vec3;
use modules::parse_obj_file;

use shared::materials;
use shared::*;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

pub fn main() {
    pretty_env_logger::init();

    let cam_data = CamData {
        transform: glam::Affine3A::IDENTITY,
        canvas_width: WIDTH as u32,
        canvas_height: HEIGHT as u32,
        fov: 90.0,
        samples: 10,
    };

    let _materials: Vec<materials::DiffuseMaterial> = vec![
        materials::DiffuseMaterial::new(Vec3::new(255.0, 0.0, 0.0)),
        materials::DiffuseMaterial::new(Vec3::new(0.0, 0.0, 255.0)),
        materials::DiffuseMaterial::new(Vec3::new(0.0, 255.0, 0.0)),
    ];

    let (teapot_vert, teapot_tris) = parse_obj_file(include_str!("./resources/spike.obj"));
    println!("teapot triangles: {:?}", teapot_tris.len());
    let teapot_vert = teapot_vert.into_boxed_slice();
    let teapot_tris = teapot_tris.into_boxed_slice();

    //let image = image::open("program/resources/earth_4.jpg").unwrap();
    //let resources = Rc::new(Resources { earth: image });
    //let normal_material = Rc::new(NormalMaterial {});
    let scene_info = SceneInfo {
        sun_orientation: Vec3::new(1.0, -1.0, 1.0),
        num_objects: 2,
    };

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Ray Tracer (Vulkan)")
        .with_inner_size(LogicalSize::new(
            cam_data.canvas_width,
            cam_data.canvas_height,
        ))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let transform_matrix = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(1.0, 1.0, 1.0),
        glam::Quat::IDENTITY,
        glam::Vec3::new(-5.0, 0.0, 10.0),
    );
    let transform_matrix_2 = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(1.0, 1.0, 1.0),
        glam::Quat::IDENTITY,
        glam::Vec3::new(5.0, 0.0, 10.0),
    );

    let teapot_object_1 = Object {
        first_triangle: 0,
        last_triangle: teapot_tris.len() as u32,
        transform: transform_matrix.inverse(),
    };
    let teapot_object_2 = Object {
        first_triangle: 0,
        last_triangle: teapot_tris.len() as u32,
        transform: transform_matrix_2.inverse(),
    };

    let mut app = unsafe {
        modules::vulkan::App::create(
            &window,
            cam_data,
            scene_info,
            teapot_vert,
            teapot_tris,
            Box::new([teapot_object_1, teapot_object_2]),
        )
        .unwrap()
    };

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
