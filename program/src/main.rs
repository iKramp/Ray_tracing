pub mod modules;
use core::f32;

use glam::{Quat, Vec3};

use shared::*;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

use crate::modules::{BufferSceneInfo, SceneBuilder};

const WIDTH: usize = 640 * 2;
const HEIGHT: usize = 360 * 2;

pub fn main() {
    pretty_env_logger::init();

    let cam_data = CamData {
        transform: glam::Affine3A::from_scale_rotation_translation(
            Vec3::ONE,
            Quat::IDENTITY,
            Vec3::new(0.0, 0.0, -20.0),
        ),
        canvas_width: WIDTH as u32,
        canvas_height: HEIGHT as u32,
        fov: 90.0,
        depth: 10,
        debug_number: 128,
        debug_information: DebugInformation::None,
        frame: 0,
        frames_without_move: 0.0,
    };

    let transform_matrix = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(0.6 * 2.0, 1.01 * 2.0, 0.6 * 2.0),
        glam::Quat::from_rotation_x(f32::consts::PI),
        glam::Vec3::new(0.0, -9.8, 0.0),
    );
    let transform_matrix_default_cube = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(5.0, 5.0, 5.0),
        glam::Quat::from_rotation_y(f32::consts::PI / 4.0),
        glam::Vec3::new(0.0, 1.9, 0.0),
    );
    let transform_matrix_dragon = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(20.0, 20.0, 20.0),
        glam::Quat::from_rotation_x(f32::consts::PI),
        glam::Vec3::new(2.0, 2.0, 0.0),
    );
    let transform_matrix_3 = glam::Affine3A::from_scale_rotation_translation(
        glam::Vec3::new(10.0, 10.0, 10.0),
        glam::Quat::IDENTITY,
        glam::Vec3::new(0.0, 0.0, 0.0),
    );

    let (scene_info, buffers) = 
        SceneBuilder::new()
            // .add_obj_file(include_str!("./resources/dragon_8k.obj"), &[transform_matrix_dragon])
            .add_obj_file(include_str!("./resources/default_cube.obj"), &[transform_matrix_default_cube])
            .add_obj_file(include_str!("./resources/cornel_box.obj"), &[transform_matrix_3])
            .add_obj_file(include_str!("./resources/teapot.obj"), &[transform_matrix])
            .sun_orientation(Vec3::new(1.0, -1.0, 1.0))
            .build();

    println!(
        "merged: {} vertices, {} triangles, {} BVH nodes",
        buffers.vertices.len(),
        buffers.triangles.len(),
        buffers.bvh.len()
    );

    let event_loop = EventLoop::new().unwrap();


    let mut winit_app = WinitApp {
        locked: false,
        frame_count: 0,
        start_time: std::time::Instant::now(),
        app: None,
        cam_data: Some(cam_data),
        scene_info: Some(scene_info),
        buffers: Some(buffers),
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
    buffers: Option<BufferSceneInfo>,
}

impl ApplicationHandler for WinitApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.app.is_none() {
            let window_attributes = Window::default_attributes()
                .with_resizable(false)
                .with_inner_size(LogicalSize::new(WIDTH as u32, HEIGHT as u32))
                .with_title("Ray Tracer (Vulkan)");
            let window = event_loop.create_window(window_attributes).unwrap();
            let mut app = unsafe {
                modules::vulkan::App::create(
                    &window,
                    self.cam_data.take().unwrap(),
                    self.scene_info.take().unwrap(),
                    self.buffers.take().unwrap(),
                )
                .unwrap()
            };
            app.cam_data.frames_without_move = 0.0;
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
                static mut PREV_CAMERA_TRANSFORM: glam::Affine3A = glam::Affine3A::IDENTITY;
                let current_camera_transform = app.cam_data.transform;
                if current_camera_transform != unsafe { PREV_CAMERA_TRANSFORM } {
                    app.cam_data.frames_without_move = 0.0;
                    unsafe { PREV_CAMERA_TRANSFORM = current_camera_transform };
                }

                self.frame_count += 1;
                let elapsed = self.start_time.elapsed().as_secs_f32();
                if elapsed > 1.0 {
                    let fps = self.frame_count as f32 / elapsed;
                    println!("FPS: {}", fps);
                    self.frame_count = 0;
                    self.start_time = std::time::Instant::now();
                }
                unsafe { app.render(window).unwrap() };
                app.cam_data.frame += 1;
                app.cam_data.frames_without_move += 1.0;
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
                    PhysicalKey::Code(KeyCode::KeyR) => {
                        app.cam_data.frames_without_move = 0.0;
                    }
                    PhysicalKey::Code(KeyCode::KeyQ) => {
                        let forward_vector = Vec3::new(0.0, -0.2, 0.0);
                        app.update_pos(forward_vector);
                    }
                    PhysicalKey::Code(KeyCode::KeyE) => {
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
                        app.cam_data.frames_without_move = 0.0;
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
                        app.cam_data.frames_without_move = 0.0;
                        if event.state == winit::event::ElementState::Released {
                            return;
                        }
                        app.cam_data.debug_number *= 2;
                        println!("debug_number: {}", app.cam_data.debug_number);
                    }
                    PhysicalKey::Code(KeyCode::NumpadSubtract) => {
                        app.cam_data.frames_without_move = 0.0;
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
