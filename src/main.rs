extern crate rand;
extern crate sdl2;
extern crate vector3d;
pub mod trace;

use hit::*;
use rand::prelude::*;
use trace::*;
use vector3d::Vector3d;

pub struct CamData {
    canvas_width: usize,
    canvas_height: usize,
    fov: f64,
    transform: Ray,
}

pub fn claculate_vec_dir_from_cam(data: &CamData, (pix_x, pix_y): (usize, usize)) -> Ray {
    //only capable up to 180 deg FOV TODO: this has to be rewritten probably. it works, but barely
    let fov_rad = data.fov / 180.0 * PI;
    let virt_canvas_height = (fov_rad / 2.0).tan();

    let pix_offset_y = (-(pix_y as f64) / data.canvas_height as f64 + 0.5) * virt_canvas_height;
    let pix_offset_x = (pix_x as f64 / data.canvas_height as f64 - 0.5) * virt_canvas_height;

    //println!("{},{}   ", pix_offset_x, pix_offset_y);

    let offset_yaw = pix_offset_x.atan();
    let offset_pitch = pix_offset_y.atan();

    let mut cam_vec = data.transform.clone();
    cam_vec.normalize();

    let mut yaw = cam_vec.orientation.x.asin();
    if cam_vec.orientation.z < 0.0 {
        yaw = PI - yaw;
    }
    let mut pitch = cam_vec.orientation.y.asin();
    if cam_vec.orientation.z < 0.0 {
        pitch = PI - pitch;
    }

    yaw += offset_yaw;
    pitch += offset_pitch;

    return trace::Ray::new(
        data.transform.pos,
        Vector3d::new(
            yaw.sin() * pitch.cos(),
            pitch.sin(),
            yaw.cos() * pitch.cos(),
        ),
    );
}

pub fn main() {
    let data = CamData {
        canvas_width: 1000,  //720
        canvas_height: 1000, //720,
        fov: 90.0,
        transform: Ray {
            pos: Vector3d::new(0.0, 0.0, 0.0),
            orientation: Vector3d::new(0.0, 0.0, 1.0),
        },
    };
    let scene_info = SceneInfo {
        sun_orientation: Vector3d::new(0.0, -1.0, 0.0),
        verts: Vec::new(),
        tris: Vec::new(),
        hittable_objects: vec![
            Box::new(Sphere::new(Vector3d::new(0.4, 0.4, 3.0), 0.5)),
            Box::new(Sphere::new(Vector3d::new(-0.4, 0.4, 5.0), 0.5)),
            Box::new(Sphere::new(Vector3d::new(0.4, -0.4, 9.0), 0.5)),
            Box::new(Sphere::new(Vector3d::new(-0.4, -0.4, 20.0), 0.5)),
        ]
    };

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "pc sim",
            data.canvas_width as u32,
            data.canvas_height as u32,
        )
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap(); //code above inits sdl2 somehow, idk what it does
    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
    canvas.clear();

    let mut rng = rand::thread_rng();

    for pix_y in 0..data.canvas_height {
        for pix_x in 0..data.canvas_width {
            let mut vec = claculate_vec_dir_from_cam(&data, (pix_x, pix_y));
            let color = vec.get_color(&scene_info);
            canvas.set_draw_color(color);
            let _res = canvas.draw_point((pix_x as i32, pix_y as i32));
        }
        println!("");
    }

    canvas.present();

    loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => return,
                _ => {}
            }
        }

        /*canvas.set_draw_color(sdl2::pixels::Color::RGB(rng.gen(), rng.gen(), rng.gen()));
        canvas.clear();
        canvas.present();*/
    }
}
