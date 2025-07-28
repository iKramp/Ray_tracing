pub mod modules;
use core::f32;

use glam::{Quat, Vec3};
use modules::bvh::create_bvh;
use modules::parse_obj_file;

use shared::materials;
use shared::*;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

pub fn main() {
    pretty_env_logger::init();

    let cam_data = CamData {
        transform: glam::Affine3A::from_scale_rotation_translation(
            Vec3::ONE,
            Quat::IDENTITY,
            Vec3::new(0.0, 0.0, -15.0),
        ),
        canvas_width: WIDTH as u32,
        canvas_height: HEIGHT as u32,
        fov: 90.0,
        samples: 1,
        depth: 5,
        debug_number: 128,
        debug_information: DebugInformation::None,
        frame: 0,
    };

    let _materials: Vec<materials::DiffuseMaterial> = vec![
        materials::DiffuseMaterial::new(Vec3::new(255.0, 0.0, 0.0)),
        materials::DiffuseMaterial::new(Vec3::new(0.0, 0.0, 255.0)),
        materials::DiffuseMaterial::new(Vec3::new(0.0, 255.0, 0.0)),
    ];

    let (teapot_vert, mut teapot_tris) = parse_obj_file(include_str!("./resources/teapot.obj"));

    // let (sandal_vert, mut sandal_tris) = parse_obj_file(include_str!("./resources/sandal.obj"));

    let (cube_vert, mut cube_tris) = parse_obj_file(include_str!("./resources/cornel_box.obj"));

    let teapot_bvh = create_bvh(teapot_vert.as_ref(), teapot_tris.as_mut());
    // let mut sandal_bvh = create_bvh(sandal_vert.as_ref(), sandal_tris.as_mut());
    let mut cube_bvh = create_bvh(cube_vert.as_ref(), cube_tris.as_mut());

    let teapot_tris_len = teapot_tris.len() as u32;
    let teapot_verts_len = teapot_vert.len() as u32;
    let teapot_bvh_len = teapot_bvh.len() as u32;
    // let sandal_tris_len = sandal_tris.len() as u32;
    // let sandal_verts_len = sandal_vert.len() as u32;
    // let sandal_bvh_len = sandal_bvh.len() as u32;

    // for triangle in sandal_tris.iter_mut() {
    //     triangle.0 += teapot_verts_len;
    //     triangle.1 += teapot_verts_len;
    //     triangle.2 += teapot_verts_len;
    // }

    for triangle in cube_tris.iter_mut() {
        triangle.0 += teapot_verts_len;//+ sandal_verts_len;
        triangle.1 += teapot_verts_len;//+ sandal_verts_len;
        triangle.2 += teapot_verts_len;//+ sandal_verts_len;
    }

    // for sandal_bvh_node in sandal_bvh.iter_mut() {
    //     if matches!(sandal_bvh_node.mode, ChildTriangleMode::Children) {
    //         sandal_bvh_node.child_1_or_first_tri += teapot_bvh_len;
    //         sandal_bvh_node.child_2_or_last_tri += teapot_bvh_len;
    //     } else {
    //         sandal_bvh_node.child_1_or_first_tri += teapot_tris_len;
    //         sandal_bvh_node.child_2_or_last_tri += teapot_tris_len;
    //     }
    // }

    for cube_bvh_node in cube_bvh.iter_mut() {
        if matches!(cube_bvh_node.mode, ChildTriangleMode::Children) {
            cube_bvh_node.child_1_or_first_tri += teapot_bvh_len;// + sandal_bvh_len;
            cube_bvh_node.child_2_or_last_tri += teapot_bvh_len;// + sandal_bvh_len;
        } else {
            cube_bvh_node.child_1_or_first_tri += teapot_tris_len;// + sandal_tris_len;
            cube_bvh_node.child_2_or_last_tri += teapot_tris_len;// + sandal_tris_len;
        }
    }

    let mut final_vert = Vec::new();
    final_vert.extend(teapot_vert);
    // final_vert.extend(sandal_vert);
    final_vert.extend(cube_vert);

    let mut final_tris = Vec::new();
    final_tris.extend(teapot_tris);
    // final_tris.extend(sandal_tris);
    final_tris.extend(cube_tris);

    let mut final_bvh = Vec::new();
    final_bvh.extend(teapot_bvh);
    // final_bvh.extend(sandal_bvh);
    final_bvh.extend(cube_bvh);

    println!(
        "merged: {} vertices, {} triangles, {} BVH nodes",
        final_vert.len(),
        final_tris.len(),
        final_bvh.len()
    );

    let scene_info = SceneInfo {
        sun_orientation: Vec3::new(1.0, -1.0, 1.0),
        num_objects: 2,
        num_bvh_nodes: final_bvh.len() as u32,
        num_triangles: final_tris.len() as u32,
    };

    let event_loop = EventLoop::new().unwrap();

    let transform_matrix = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(1.0, 1.7, 1.0),
        glam::Quat::from_rotation_x(f32::consts::PI),
        glam::Vec3::new(-2.0, 2.0, 0.0),
    );
    let transform_matrix_2 = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(1.0, 1.0, 1.0),
        glam::Quat::from_rotation_x(f32::consts::PI / 3.0 * 4.0)
            * glam::Quat::from_rotation_y(f32::consts::PI / 5.0 * 3.0),
        glam::Vec3::new(2.0, 2.0, 5.0),
    );

    let transform_matrix_3 = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(20.0, 20.0, 20.0),
        glam::Quat::IDENTITY,
        glam::Vec3::new(0.0, 0.0, 0.0),
    );

    let teapot_object = Object { bvh_root: 0 };
    let sandal_object = Object {
        bvh_root: teapot_bvh_len as u32,
    };
    let cube_object = Object {
        bvh_root: teapot_bvh_len as u32,// + sandal_bvh_len as u32,
    };

    let teapot_instance = Instance {
        transform: transform_matrix.inverse(),
        object_id: 0,
    };

    let sandal_instance = Instance {
        transform: transform_matrix_2.inverse(),
        object_id: 1,
    };

    let cube_instance = Instance {
        transform: transform_matrix_3.inverse(),
        object_id: 2,
    };

    let instances = Box::new([
        teapot_instance,
        // sandal_instance,
        cube_instance,
    ]);
    assert!(instances.len() == scene_info.num_objects as usize);

    let mut winit_app = WinitApp {
        locked: false,
        frame_count: 0,
        start_time: std::time::Instant::now(),
        app: None,
        cam_data: Some(cam_data),
        scene_info: Some(scene_info),
        vertex_buffer: Some(final_vert.into_boxed_slice()),
        triangle_buffer: Some(final_tris.into_boxed_slice()),
        object_buffer: Some(Box::new([teapot_object, sandal_object, cube_object])),
        instance_buffer: Some(instances),
        bvh_buffer: Some(final_bvh.into_boxed_slice()),
    };

    let _res = event_loop.run_app(&mut winit_app);
}

struct WinitApp {
    locked: bool,
    frame_count: usize,
    start_time: std::time::Instant,
    app: Option<(modules::vulkan::App, winit::window::Window)>,
    cam_data: Option<CamData>,
    scene_info: Option<SceneInfo>,
    vertex_buffer: Option<Box<[Vertex]>>,
    triangle_buffer: Option<Box<[(u32, u32, u32)]>>,
    object_buffer: Option<Box<[Object]>>,
    instance_buffer: Option<Box<[Instance]>>,
    bvh_buffer: Option<Box<[Bvh]>>,
}

impl ApplicationHandler for WinitApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.app.is_none() {
            let window_attributes = Window::default_attributes()
                .with_resizable(false)
                .with_inner_size(LogicalSize::new(WIDTH as u32, HEIGHT as u32))
                .with_title("Ray Tracer (Vulkan)");
            let window = event_loop.create_window(window_attributes).unwrap();
            let app = unsafe {
                modules::vulkan::App::create(
                    &window,
                    self.cam_data.take().unwrap(),
                    self.scene_info.take().unwrap(),
                    self.vertex_buffer.take().unwrap(),
                    self.triangle_buffer.take().unwrap(),
                    self.object_buffer.take().unwrap(),
                    self.instance_buffer.take().unwrap(),
                    self.bvh_buffer.take().unwrap(),
                )
                .unwrap()
            };
            self.app = Some((app, window));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::RedrawRequested = event {
            if let Some((app, window)) = &mut self.app {
                self.frame_count += 1;
                let elapsed = self.start_time.elapsed().as_secs_f32();
                if elapsed > 1.0 {
                    let fps = self.frame_count as f32 / elapsed;
                    println!("FPS: {}", fps);
                    self.frame_count = 0;
                    self.start_time = std::time::Instant::now();
                }
                println!("frame: {}", app.cam_data.frame);
                unsafe { app.render(window).unwrap() };
                app.cam_data.frame += 1;
            }
        } else if let WindowEvent::CloseRequested = event {
            if let Some((_app, _window)) = &mut self.app {
                unsafe {
                    if let Some((mut app, _window)) = self.app.take() {
                        app.destroy();
                    }
                }
                event_loop.exit()
            }
        } else if let WindowEvent::KeyboardInput { event, .. } = event {
            if let Some((app, window)) = &mut self.app {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => {
                        let forward_vector = Vec3::new(0.0, 0.0, 0.2);
                        let (_scale, rotation, _translation) =
                            app.cam_data.transform.to_scale_rotation_translation();
                        let (yaw, _, _) = rotation.to_euler(glam::EulerRot::YXZ);
                        let horizontal_rotation = Quat::from_rotation_y(yaw);
                        let forward_vector = horizontal_rotation * forward_vector;
                        app.update_pos(forward_vector);
                    }
                    PhysicalKey::Code(KeyCode::KeyS) => {
                        let forward_vector = Vec3::new(0.0, 0.0, -0.2);
                        let (_scale, rotation, _translation) =
                            app.cam_data.transform.to_scale_rotation_translation();
                        let (yaw, _, _) = rotation.to_euler(glam::EulerRot::YXZ);
                        let horizontal_rotation = Quat::from_rotation_y(yaw);
                        let forward_vector = horizontal_rotation * forward_vector;
                        app.update_pos(forward_vector);
                    }
                    PhysicalKey::Code(KeyCode::KeyA) => {
                        let forward_vector = Vec3::new(-0.2, 0.0, 0.0);
                        let (_scale, rotation, _translation) =
                            app.cam_data.transform.to_scale_rotation_translation();
                        let forward_vector = rotation * forward_vector;
                        app.update_pos(forward_vector);
                    }
                    PhysicalKey::Code(KeyCode::KeyD) => {
                        let forward_vector = Vec3::new(0.2, 0.0, 0.0);
                        let (_scale, rotation, _translation) =
                            app.cam_data.transform.to_scale_rotation_translation();
                        let forward_vector = rotation * forward_vector;
                        app.update_pos(forward_vector);
                    }
                    PhysicalKey::Code(KeyCode::Space) => {
                        let forward_vector = Vec3::new(0.0, -0.2, 0.0);
                        app.update_pos(forward_vector);
                    }
                    PhysicalKey::Code(KeyCode::ShiftLeft) => {
                        let forward_vector = Vec3::new(0.0, 0.2, 0.0);
                        app.update_pos(forward_vector);
                    }
                    PhysicalKey::Code(KeyCode::KeyL) => {
                        if event.state == winit::event::ElementState::Released {
                            return;
                        }
                        self.locked = !self.locked;
                        if self.locked {
                            window
                                .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                                .unwrap();
                        } else {
                            window
                                .set_cursor_grab(winit::window::CursorGrabMode::None)
                                .unwrap();
                        }
                    }
                    PhysicalKey::Code(KeyCode::Enter) => {
                        if event.state == winit::event::ElementState::Released {
                            return;
                        }
                        app.cam_data.debug_information = match app.cam_data.debug_information {
                            DebugInformation::None => DebugInformation::TriangleIntersection,
                            DebugInformation::TriangleIntersection => {
                                DebugInformation::BvhIntersection
                            }
                            DebugInformation::BvhIntersection => DebugInformation::None,
                        };
                        println!(
                            "debug information: {:?}",
                            app.cam_data.debug_information
                        );
                    }
                    PhysicalKey::Code(KeyCode::NumpadAdd) => {
                        if event.state == winit::event::ElementState::Released {
                            return;
                        }
                        app.cam_data.debug_number *= 2;
                        println!("debug_number: {}", app.cam_data.debug_number);
                    }
                    PhysicalKey::Code(KeyCode::NumpadSubtract) => {
                        if event.state == winit::event::ElementState::Released {
                            return;
                        }
                        app.cam_data.debug_number /= 2;
                        println!("debug number: {}", app.cam_data.debug_number);
                    }

                    _ => {}
                }
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let winit::event::DeviceEvent::MouseMotion { delta } = event {
            if let Some((app, _window)) = &mut self.app {
                if !self.locked {
                    return;
                }
                app.update_mouse(delta.0 as f32, delta.1 as f32);
            }
        }
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        unsafe {
            if let Some((mut app, _window)) = self.app.take() {
                app.destroy();
            }
        }
    }
}
