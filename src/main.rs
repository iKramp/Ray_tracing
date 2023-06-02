extern crate sdl2;
extern crate rand;

use rand::prelude::*;

#[derive(Clone, Copy)]
pub struct Pos {
    x: f64,
    y: f64,
    z: f64
}

pub struct CamVector {
    pos: Pos,
    pitch: f64,
    yaw: f64
}

pub struct CamData {
    canvas_width: usize,
    canvas_height: usize,
    fov: f64,
    pos: Pos,
    rotation_pitch: f64,
    rotation_yaw: f64
}

pub fn claculate_vec(data: &CamData, (pix_x, pix_y): (usize, usize)) -> CamVector {
    let offset_x = pix_x as f64 / data.canvas_width as f64 - 0.5;
    let offset_y = -(pix_y as f64) / data.canvas_width as f64 + 0.5; //also width so we get uniform scaling, aka when the window is squished we don't want the y axis to angle as much as the x axis
    let offset_x = offset_x * data.fov;
    let offset_y = offset_y * data.fov;

    return CamVector{ pos: data.pos, pitch: data.rotation_pitch + offset_y, yaw: data.rotation_yaw + offset_x};
}


pub fn main() {
    let data = CamData {
        canvas_width: 1000,
        canvas_height: 1000,
        fov: 90.0,
        pos: Pos { x: 0.0, y: 0.0, z: 0.0 },
        rotation_pitch: 45.0,
        rotation_yaw: 45.0,
    };


    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("pc sim", data.canvas_width as u32, data.canvas_height as u32)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();//code above inits sdl2 somehow, idk what it does
    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
    canvas.clear();

    let mut rng = rand::thread_rng();

    for pix_y in 0..data.canvas_height {
        for pix_x in 0..data.canvas_width {
            let vec = claculate_vec(&data, (pix_x, pix_y));
            canvas.set_draw_color(sdl2::pixels::Color::RGB((vec.pitch / 90.0 * 255.0) as u8, (vec.yaw / 90.0 * 255.0) as u8, 0));
            let _res = canvas.draw_point((pix_x as i32, pix_y as i32));
        }
    }

    canvas.present();

    loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => return,
                _ => {}
            }
        }

        /*canvas.set_draw_color(sdl2::pixels::Color::RGB(rng.gen(), rng.gen(), rng.gen()));
        canvas.clear();
        canvas.present();*/
    }


}