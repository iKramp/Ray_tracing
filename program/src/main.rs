pub mod modules;
use core::f32;

use glam::{Quat, Vec3};

use shared::*;
use tracer::modules::material::GenericMaterial;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

use crate::modules::point_recorder::fake_points;
use crate::modules::{record_points, scene_builder::SceneBuilder};

const WIDTH: usize = 640 * 2;
const HEIGHT: usize = 360 * 2;

pub fn main() {
    pretty_env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    let save_buffers = args.iter().any(|arg| arg == "--save-buffers");

    let cam_data = CamData {
        transform: glam::Affine3A::from_scale_rotation_translation(
            Vec3::ONE,
            Quat::IDENTITY,
            Vec3::new(0.0, 0.0, -20.0),
        ),
        canvas_width: WIDTH as u32,
        canvas_height: HEIGHT as u32,
        fov: 65.0,
        depth: 10,
        debug_number: 128,
        debug_information: DebugInformation::None,
        frame: 0,
        reset: 1,
        random_seed: 0xDEADBEEF,
        debug_point_color: Vec3Aligned::new(Vec3::ZERO),
    };

    let transform_matrix_teapot_light = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(0.6 * 4.0, 1.01 * 2.0, 0.6 * 4.0),
        glam::Quat::from_rotation_x(f32::consts::PI),
        glam::Vec3::new(0.0, -9.8, 0.0),
    );
    let transform_matrix_dragon = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(15.0, 15.0, 15.0),
        glam::Quat::from_rotation_x(f32::consts::PI)
            * glam::Quat::from_rotation_y(f32::consts::PI / 2.0),
        glam::Vec3::new(0.0, 2.0, 0.0),
    );
    let transform_matrix_sphere = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(5.0, 5.0, 5.0),
        glam::Quat::IDENTITY,
        glam::Vec3::new(0.0, 2.0, 0.0),
    );
    let transform_matrix_cornel_box = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(10.0, 10.0, 10.0),
        glam::Quat::IDENTITY,
        glam::Vec3::new(0.0, 0.0, 0.0),
    );
    let transform_matrix_plane_light = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(3.0, 3.0, 3.0),
        glam::Quat::from_rotation_x(f32::consts::PI),
        glam::Vec3::new(0.0, -9.99, 0.0),
    );
    let transform_matrix_cornel_plane_down = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(10.0, 10.0, 10.0),
        glam::Quat::IDENTITY,
        glam::Vec3::new(0.0, 10.0, 0.0),
    );
    let transform_matrix_cornel_plane_up = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(10.0, 10.0, 10.0),
        glam::Quat::from_rotation_x(f32::consts::PI),
        glam::Vec3::new(0.0, -10.0, 0.0),
    );
    let transform_matrix_cornel_plane_left = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(10.0, 10.0, 10.0),
        glam::Quat::from_rotation_z(f32::consts::PI / 2.0),
        glam::Vec3::new(-10.0, 0.0, 0.0),
    );
    let transform_matrix_cornel_plane_right = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(10.0, 10.0, 10.0),
        glam::Quat::from_rotation_z(-f32::consts::PI / 2.0),
        glam::Vec3::new(10.0, 0.0, 0.0),
    );
    let transform_matrix_cornel_plane_front = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(10.0, 10.0, 10.0),
        glam::Quat::from_rotation_x(-f32::consts::PI / 2.0),
        glam::Vec3::new(0.0, 0.0, -10.0),
    );
    let transform_matrix_cornel_plane_back = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(10.0, 10.0, 10.0),
        glam::Quat::from_rotation_x(f32::consts::PI / 2.0),
        glam::Vec3::new(0.0, 0.0, 10.0),
    );

    let (scene_info, mut buffers) = SceneBuilder::new()
        .add_material(GenericMaterial {
            color_surface: Vec3::new(0.9, 0.9, 0.9),
            color_emissive: Vec3::ZERO,
            specular: 0.0,
            specular_roughness: 0.0,
            roughness: 0.7,
            ior: 0.0,
            padding_1: [0; 4],
            padding_2: [0; 4],
        })
        .add_material(GenericMaterial {
            color_surface: Vec3::new(0.0, 0.0, 0.0),
            color_emissive: Vec3::new(12.0, 15.0, 15.0) * 0.7,
            specular: 0.0,
            specular_roughness: 0.0,
            roughness: 0.0,
            ior: 0.0, //no refraction
            padding_1: [0; 4],
            padding_2: [0; 4],
        })
        .add_material(GenericMaterial {
            color_surface: Vec3::new(0.9, 0.9, 0.9),
            color_emissive: Vec3::ZERO,
            specular: 0.0,
            specular_roughness: 0.0,
            roughness: 0.9,
            ior: 0.0, //no refraction
            padding_1: [0; 4],
            padding_2: [0; 4],
        })
        .add_material(GenericMaterial {
            color_surface: Vec3::new(0.7, 0.075, 0.075),
            color_emissive: Vec3::ZERO,
            specular: 0.0,
            specular_roughness: 0.0,
            roughness: 0.9,
            ior: 0.0, //no refraction
            padding_1: [0; 4],
            padding_2: [0; 4],
        })
        .add_material(GenericMaterial {
            color_surface: Vec3::new(0.075, 0.075, 0.7),
            color_emissive: Vec3::ZERO,
            specular: 0.0,
            specular_roughness: 0.0,
            roughness: 0.9,
            ior: 0.0, //no refraction
            padding_1: [0; 4],
            padding_2: [0; 4],
        })
        .add_material(GenericMaterial {
            color_surface: Vec3::new(0.075, 0.7, 0.075),
            color_emissive: Vec3::ZERO,
            specular: 0.0,
            specular_roughness: 0.0,
            roughness: 0.9,
            ior: 0.0, //no refraction
            padding_1: [0; 4],
            padding_2: [0; 4],
        })
        // .add_obj_file(
        //     include_str!("./resources/dragon_8k.obj"),
        //     &[transform_matrix_dragon],
        //     0,
        // )
        // .add_obj_file(
        //     include_str!("./resources/smooth_sphere.obj"),
        //     &[transform_matrix_sphere],
        //     0,
        // )
        .add_obj_file(
            include_str!("./resources/ico_sphere_sharp.obj"),
            &[transform_matrix_sphere],
            0,
        )
        // .add_obj_file(
        //     include_str!("./resources/plane.obj"),
        //     &[transform_matrix_cornel_plane_down],
        //     2,
        // )
        // .add_obj_file(
        //     include_str!("./resources/plane.obj"),
        //     &[transform_matrix_cornel_plane_up],
        //     2,
        // )
        // .add_obj_file(
        //     include_str!("./resources/plane.obj"),
        //     &[transform_matrix_cornel_plane_left],
        //     4,
        // )
        // .add_obj_file(
        //     include_str!("./resources/plane.obj"),
        //     &[transform_matrix_cornel_plane_right],
        //     5,
        // )
        // .add_obj_file(
        //     include_str!("./resources/plane.obj"),
        //     &[transform_matrix_cornel_plane_front],
        //     2,
        // )
        // .add_obj_file(
        //     include_str!("./resources/plane.obj"),
        //     &[transform_matrix_cornel_plane_back],
        //     3,
        // )
        .add_obj_file(
            include_str!("./resources/plane.obj"),
            &[transform_matrix_plane_light],
            1,
        )
        .sun_orientation(Vec3::new(1.0, -1.0, 1.0))
        .build();

    if save_buffers {
        buffers.print();
        buffers.save_buffers("buffers_dump.bin");
        return;
    }

    fake_points(&mut buffers, &cam_data);

    let event_loop = EventLoop::new().unwrap();

    let window_attributes = Window::default_attributes()
        .with_resizable(false)
        .with_inner_size(LogicalSize::new(WIDTH as u32, HEIGHT as u32))
        .with_title("Ray Tracer (Vulkan)");
    let window = event_loop.create_window(window_attributes).unwrap();

    let mut vulkan_app =
        unsafe { modules::vulkan::App::create(&window, cam_data, scene_info, buffers).unwrap() };
    vulkan_app.cam_data.reset = 1;

    let mut winit_app = WinitApp {
        locked: false,
        mouse_pos_px: (0, 0),
        frame_count: 0,
        start_time: std::time::Instant::now(),
        app: (vulkan_app, window),
    };

    let _res = event_loop.run_app(&mut winit_app);
}

struct WinitApp {
    locked: bool,
    mouse_pos_px: (u32, u32),
    frame_count: usize,
    start_time: std::time::Instant,
    app: (modules::vulkan::App, winit::window::Window),
}

impl ApplicationHandler for WinitApp {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::RedrawRequested = event {
            let (app, window) = &mut self.app;
            static mut PREV_CAMERA_TRANSFORM: glam::Affine3A = glam::Affine3A::IDENTITY;
            let current_camera_transform = app.cam_data.transform;
            if current_camera_transform != unsafe { PREV_CAMERA_TRANSFORM } {
                app.cam_data.reset = 1;
                unsafe { PREV_CAMERA_TRANSFORM = current_camera_transform };
            }

            self.frame_count += 1;
            if app.inc_rand {
                app.cam_data.random_seed = app.cam_data.random_seed.wrapping_mul(0x9E37_79B9);
            }
            let elapsed = self.start_time.elapsed().as_secs_f32();
            if elapsed > 1.0 {
                let fps = self.frame_count as f32 / elapsed;
                println!("FPS: {fps}");
                self.frame_count = 0;
                self.start_time = std::time::Instant::now();
            }
            unsafe { app.render(window).unwrap() };
            app.cam_data.reset = 0;
            app.cam_data.frame += 1;
        } else if let WindowEvent::CloseRequested = event {
            let (_app, _window) = &mut self.app;
            unsafe {
                self.app.0.destroy();
            }
            event_loop.exit()
        } else if let WindowEvent::KeyboardInput { event, .. } = event {
            let (app, window) = &mut self.app;
            if event.state == winit::event::ElementState::Released {
                return;
            }
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
                PhysicalKey::Code(KeyCode::KeyR) => {
                    app.cam_data.reset = 1;
                }
                PhysicalKey::Code(KeyCode::KeyQ) => {
                    let forward_vector = Vec3::new(0.0, -0.2, 0.0);
                    app.update_pos(forward_vector);
                }
                PhysicalKey::Code(KeyCode::KeyE) => {
                    let forward_vector = Vec3::new(0.0, 0.2, 0.0);
                    app.update_pos(forward_vector);
                }
                PhysicalKey::Code(KeyCode::KeyP) => {
                    let app = &mut self.app.0;
                    record_points(
                        &mut app.buffers,
                        self.mouse_pos_px,
                        &mut app.cam_data,
                        &app.scene_info,
                    );
                    app.cam_data.reset = 1;
                    app.dbg_ray = Some((app.cam_data.transform, self.mouse_pos_px));
                }
                PhysicalKey::Code(KeyCode::KeyI) => {
                    self.app.0.inc_rand = !self.app.0.inc_rand;
                }
                PhysicalKey::Code(KeyCode::KeyK) => {
                    let app = &mut self.app.0;
                    app.cam_data.random_seed =
                        app.cam_data.random_seed.overflowing_mul(0x9E37_79B9).0;

                    //recalculate debug points
                    if let Some((dbg_transform, coords)) = app.dbg_ray {
                        let curr_transform = app.cam_data.transform;
                        app.cam_data.transform = dbg_transform;
                        record_points(&mut app.buffers, coords, &mut app.cam_data, &app.scene_info);
                        app.cam_data.transform = curr_transform;
                        app.cam_data.reset = 1;
                        app.cam_data.transform = curr_transform;
                    }
                }
                PhysicalKey::Code(KeyCode::KeyL) => {
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
                    app.cam_data.reset = 1;
                    if event.state == winit::event::ElementState::Released {
                        return;
                    }
                    app.cam_data.debug_information = match app.cam_data.debug_information {
                        DebugInformation::None => DebugInformation::TriangleIntersection,
                        DebugInformation::TriangleIntersection => DebugInformation::BvhIntersection,
                        DebugInformation::BvhIntersection => DebugInformation::None,
                        DebugInformation::RecordPoints => DebugInformation::None,
                    };
                    println!("debug information: {:?}", app.cam_data.debug_information);
                }
                PhysicalKey::Code(KeyCode::NumpadAdd) => {
                    app.cam_data.reset = 1;
                    if event.state == winit::event::ElementState::Released {
                        return;
                    }
                    app.cam_data.debug_number *= 2;
                    println!("debug_number: {}", app.cam_data.debug_number);
                }
                PhysicalKey::Code(KeyCode::NumpadSubtract) => {
                    app.cam_data.reset = 1;
                    if event.state == winit::event::ElementState::Released {
                        return;
                    }
                    app.cam_data.debug_number /= 2;
                    println!("debug number: {}", app.cam_data.debug_number);
                }

                _ => {}
            }
        } else if let WindowEvent::CursorMoved { position, .. } = event {
            self.mouse_pos_px = (position.x as u32, position.y as u32);
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let winit::event::DeviceEvent::MouseMotion { delta } = event {
            let (app, _window) = &mut self.app;
            if !self.locked {
                return;
            }
            app.update_mouse(delta.0 as f32, delta.1 as f32);
        }
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        unsafe {
            self.app.0.destroy();
        }
    }
}
