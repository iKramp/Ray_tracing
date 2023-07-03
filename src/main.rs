pub mod ray;
use rand::prelude::*;
use ray::{data::*, material::*, trace::*};
use std::rc::Rc;
use vector3d::Vector3d;

const SHADER: &[u8] = include_bytes!(env!("shader.spv"));

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
            let mut color = Vector3d::new(0.0, 0.0, 0.0);
            for _i in 0..data.samples {
                let mut vec = claculate_vec_dir_from_cam(
                    &data,
                    (
                        pix_x as f64 + rng.gen_range(0.0..1.0),
                        pix_y as f64 + rng.gen_range(0.0..1.0),
                    ),
                );
                color = color + vec.trace_ray(&scene_info, 0, &mut rng, resources.clone());
                let _res = canvas.draw_point((pix_x as i32, pix_y as i32));
            }
            color = color / data.samples as f64 / 256.0;
            color.x = color.x.sqrt().clamp(0.0, 0.999999999);
            color.y = color.y.sqrt().clamp(0.0, 0.999999999);
            color.z = color.z.sqrt().clamp(0.0, 0.999999999);
            color = color * 256.0;
            canvas.set_draw_color(sdl2::pixels::Color::RGB(
                color.x as u8,
                color.y as u8,
                color.z as u8,
            ));
        }
        println!(
            "estimated time left: {}s",
            (std::time::Instant::now() - start).as_secs_f64() / (pix_y as f64 + 1.0)
                * (data.canvas_height - pix_y) as f64
        );
        canvas.present();
        for event in event_pump.poll_iter() {
            if let sdl2::event::Event::Quit { .. } = event {
                return;
            }
        }
    }

    canvas.present();

    loop {
        for event in event_pump.poll_iter() {
            if let sdl2::event::Event::Quit { .. } = event {
                return;
            }
        }
    }
}
