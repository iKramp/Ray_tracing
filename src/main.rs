extern crate rand;
extern crate sdl2;
extern crate vector3d;
pub mod trace;

use hit::*;
use rand::prelude::*;
use trace::*;
use vector3d::Vector3d;
use std::rc::Rc;

fn col_from_frac(r: f64, g: f64, b: f64) -> Vector3d<f64> {
    Vector3d::new(r * 255.0, g * 255.0, b * 255.0)
}

pub struct CamData {
    canvas_width: usize,
    canvas_height: usize,
    fov: f64,
    transform: Ray,
    samples: u32,
}


pub fn claculate_vec_dir_from_cam(data: &CamData, (pix_x, pix_y): (f64, f64)) -> Ray {
    //only capable up to 180 deg FOV TODO: this has to be rewritten probably. it works, but barely
    let fov_rad = data.fov / 180.0 * PI;
    let virt_canvas_height = (fov_rad / 2.0).tan();

    let pix_offset_y = (-pix_y / data.canvas_height as f64 + 0.5) * virt_canvas_height;
    let pix_offset_x = ( pix_x / data.canvas_height as f64 - 0.5 * (data.canvas_width as f64 / data.canvas_height as f64)) * virt_canvas_height;

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
        canvas_width: 498,  //1280
        canvas_height: 280, //720,
        fov: 70.0,
        transform: Ray {
            pos: Vector3d::new(0.0, 0.2, -2.0),
            orientation: Vector3d::new(0.0, 0.0, 1.0),
        },
        samples: 30,
    };
    //let diffuse_white = std::rc::Rc::new(DiffuseMaterial::new(Vector3d::new(128.0, 128.0, 128.0)));
    //let metal_dark = std::rc::Rc::new(MetalMaterial::new(Vector3d::new(20.0, 20.0, 20.0)));
    //let diffuse_red = std::rc::Rc::new(DiffuseMaterial::new(Vector3d::new(245.0, 17.0, 43.0)));

    let material_ground = Rc::new(DiffuseMaterial::new(col_from_frac(0.8, 0.8, 0.0)));
    let material_center = Rc::new(DiffuseMaterial::new(col_from_frac(0.7, 0.3, 0.3)));
    let material_left = Rc::new(MetalMaterial::new(col_from_frac(0.8, 0.8, 0.8), 0.3));
    let material_right = Rc::new(MetalMaterial::new(col_from_frac(0.8, 0.6, 0.2), 1.0));

    let scene_info = SceneInfo {
        sun_orientation: Vector3d::new(1.0, -1.0, 1.0),
        hittable_objects: vec![
            //Box::new(Sphere::new(Vector3d::new(0.0, -1.0, 5.0), 1.0, Box::new(diffuse_red.clone()))),
            //Box::new(Sphere::new(Vector3d::new(1.0, -1.6, 4.0), 0.3, Box::new(metal_dark.clone()))),
            //Box::new(Sphere::new(Vector3d::new(0.0, -1002.0, 0.0), 1000.0, Box::new(diffuse_white.clone()))),

        Box::new(Sphere::new(Vector3d::new( 0.0, -100.5, 1.0), 100.0, Box::new(material_ground.clone()))),
        Box::new(Sphere::new(Vector3d::new( 0.0,    0.0, 1.0),   0.5, Box::new(material_center.clone()))),
        Box::new(Sphere::new(Vector3d::new(-1.0,    0.0, 1.0),   0.5, Box::new(material_left  .clone()))),
        Box::new(Sphere::new(Vector3d::new( 1.0,    0.0, 1.0),   0.5, Box::new(material_right .clone()))),
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
            let mut color = Vector3d::new(0.0, 0.0, 0.0);
            for i in 0..data.samples {
                let mut vec = claculate_vec_dir_from_cam(&data, (pix_x as f64 + rng.gen_range(0.0..1.0), pix_y as f64 + rng.gen_range(0.0..1.0)));
                color = color + vec.trace_ray(&scene_info, 0, &mut rng);
                let _res = canvas.draw_point((pix_x as i32, pix_y as i32));
            }
            color = color / data.samples as f64 / 256.0;
            color.x = color.x.sqrt().clamp(0.0, 0.999999999);
            color.y = color.y.sqrt().clamp(0.0, 0.999999999);
            color.z = color.z.sqrt().clamp(0.0, 0.999999999);
            color = color * 256.0;
            canvas.set_draw_color(sdl2::pixels::Color::RGB(color.x as u8, color.y as u8, color.z as u8));
        }
        //println!("{}/{}", pix_y, data.canvas_height);
        canvas.present();
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => return,
                _ => {}
            }
        }
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
