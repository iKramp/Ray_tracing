extern crate rand;
extern crate sdl2;
pub mod trace;

use rand::prelude::*;
use trace::*;

pub struct CamData {
    canvas_width: usize,
    canvas_height: usize,
    fov: f64,
    transform: Ray,
}

pub fn claculate_vec_dir_from_cam(data: &CamData, (pix_x, pix_y): (usize, usize)) -> Ray {
    let offset_yaw = (pix_x as f64 / data.canvas_width as f64 - 0.5) * data.fov;
    let offset_pitch = (-(pix_y as f64) / data.canvas_width as f64 + 0.5) * data.fov; //also width so we get uniform scaling, aka when the window is squished we don't want the y axis to angle as much as the x axis
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

    yaw += offset_yaw * PI / 180.0;
    pitch += offset_pitch * PI / 180.0;

    return trace::Ray {
        pos: data.transform.pos,
        orientation: Vec3 {
            x: yaw.sin() * pitch.cos(),
            y: pitch.sin(),
            z: yaw.cos() * pitch.cos(),
        },
    };
}

pub fn main() {
    let data = CamData {
        canvas_width: 1280,
        canvas_height: 1280,//720,
        fov: 90.0,
        transform: Ray {
            pos: Vec3::new(0.0, 0.0, 0.0),
            orientation: Vec3::new(0.0, 0.0, 1.0),
        },
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
            let color = vec.get_color();
            if pix_x == 0 {
                print!("{:.1},{:.1},{:.1}   ", vec.orientation.x, vec.orientation.y, vec.orientation.z);
                print!("{:.1},{:.1},{:.1}   ", color.r, color.g, color.b);
            }
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
