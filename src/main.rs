pub mod ray;
use rand::prelude::*;
use ray::{data::*, material::*, trace::*};
use std::rc::Rc;
use vector3d::Vector3d;

const SHADER: &[u8] = include_bytes!(env!("shader.spv"));

fn check_for_quit(event_pump: &mut sdl2::EventPump) {
    for event in event_pump.poll_iter() {
        if let sdl2::event::Event::Quit { .. } = event {
            return;
        }
    }
}

pub fn main() {
    let data = CamData::default();

    let image = image::open("src/resources/earth_4.jpg").unwrap();
    let resources = Rc::new(Resources { earth: image });
    //default materials
    let _normal_material = Rc::new(NormalMaterial {});

    let scene_info = SceneInfo::default();

    println!("rendering...");

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "ray tracer",
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

    let start = std::time::Instant::now();

    for pix_y in 0..data.canvas_height {
        for pix_x in 0..data.canvas_width {
            let color = Ray::get_color((pix_x, pix_y), &mut rng, &data, &scene_info, &resources);
            canvas.set_draw_color(sdl2::pixels::Color::RGB(
                color.x as u8,
                color.y as u8,
                color.z as u8,
            ));
            let _res = canvas.draw_point((pix_x as i32, pix_y as i32));
        }
        println!(
            "estimated time left: {}s",
            (std::time::Instant::now() - start).as_secs_f64() / (pix_y as f64 + 1.0)
                * (data.canvas_height - pix_y) as f64
        );
        canvas.present();
        check_for_quit(&mut event_pump);
    }

    canvas.present();

    loop {
        check_for_quit(&mut event_pump);
        std::thread::sleep(std::time::Duration::from_millis(100))
    }
}
