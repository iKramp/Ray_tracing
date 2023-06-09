extern crate image;
extern crate rand;
extern crate sdl2;
extern crate vector3d;
pub mod ray;

use core::f64::consts::PI;
use rand::prelude::*;
use ray::{hit::*, material::*, trace::*};
use std::rc::Rc;
use vector3d::Vector3d;


const SHADER: &[u8] = include_bytes!(env!("shader.spv"));



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

pub struct Resources {
    earth: image::DynamicImage,
}

pub fn claculate_vec_dir_from_cam(data: &CamData, (pix_x, pix_y): (f64, f64)) -> Ray {
    //only capable up to 180 deg FOV TODO: this has to be rewritten probably. it works, but barely
    let fov_rad = data.fov / 180.0 * PI;
    let virt_canvas_height = (fov_rad / 2.0).tan();

    let pix_offset_y = (-pix_y / data.canvas_height as f64 + 0.5) * virt_canvas_height;
    let pix_offset_x = (pix_x / data.canvas_height as f64
        - 0.5 * (data.canvas_width as f64 / data.canvas_height as f64))
        * virt_canvas_height;

    //println!("{},{}   ", pix_offset_x, pix_offset_y);

    let offset_yaw = pix_offset_x.atan();
    let offset_pitch = pix_offset_y.atan();

    let mut cam_vec = data.transform;
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

    Ray::new(
        data.transform.pos,
        Vector3d::new(
            yaw.sin() * pitch.cos(),
            pitch.sin(),
            yaw.cos() * pitch.cos(),
        ),
    )
}

fn parse_obj_file(path: &str, scale: f64, transform: Vector3d<f64>) -> Box<dyn HitObject> {
    let _objects: Vec<Box<dyn HitObject>> = Vec::new();
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut faces: Vec<(usize, usize, usize)> = Vec::new();

    let file = std::fs::read_to_string(path).unwrap();
    for line in file.lines() {
        let mut line = line.split_whitespace();
        match line.next() {
            Some("v") => {
                let x = line.next().unwrap().parse::<f64>().unwrap() * scale + transform.x;
                let y = line.next().unwrap().parse::<f64>().unwrap() * scale + transform.y;
                let z = line.next().unwrap().parse::<f64>().unwrap() * scale + transform.z;
                vertices.push(Vertex::new(Vector3d::new(x, y, z), (0.0, 0.0)));
            }
            Some("f") => {
                let v1 = line.next().unwrap().parse::<usize>().unwrap() - 1;
                let v2 = line.next().unwrap().parse::<usize>().unwrap() - 1;
                let v3 = line.next().unwrap().parse::<usize>().unwrap() - 1;
                faces.push((v1, v2, v3));
            }
            _ => {}
        }
    }

    Box::new(
          Mesh::new(
              vertices,
              faces,
              Box::new(Rc::new(NormalMaterial{})),
          )
    )
}

pub fn main() {
    let data = CamData {
        canvas_width: 247,//498,  //1280, 498
        canvas_height: 140,//280, //720, 280
        fov: 30.0,
        transform: Ray {
            pos: Vector3d::new(0.0, 0.5, -5.0),
            orientation: Vector3d::new(0.0, 0.0, 1.0),
        },
        samples: 1,
    };

    let resources;
    let image = image::open("src/resources/earth_4.jpg");
    match image {
        Err(e) => {
            println!("{}", e);
            panic!()
        },
        Ok(image) => {
            resources = Rc::new(Resources {
                earth: image,
            });
        }
    }
    //default materials
    let _normal_material = Rc::new(NormalMaterial{});


    /*nice scene with different materials
    let material_ground = Rc::new(MetalMaterial::new(col_from_frac(0.8, 0.8, 0.0), 0.0));
    let material_center = Rc::new(EmmissiveMaterial::new(col_from_frac(1.0, 1.0, 1.0)));
    let material_left = Rc::new(MetalMaterial::new(col_from_frac(0.8, 0.8, 0.8), 0.3));
    let material_right = Rc::new(MetalMaterial::new(col_from_frac(0.8, 0.6, 0.2), 1.0));
    */

    //color box, dimensions 1x1x1, centered at 0,0.5,0
    let _material_ground = Rc::new(DiffuseMaterial::new(col_from_frac(0.0, 1.0, 0.0)));
    let _material_diffuse = Rc::new(DiffuseMaterial::new(col_from_frac(0.8, 0.8, 0.8)));
    let material_metal = Rc::new(MetalMaterial::new(col_from_frac(0.8, 0.8, 0.8), 0.05));
    let material_left = Rc::new(DiffuseMaterial::new(col_from_frac(1.0, 0.0, 0.0)));
    let material_right = Rc::new(DiffuseMaterial::new(col_from_frac(0.0, 0.0, 1.0)));
    let material_back_front = Rc::new(DiffuseMaterial::new(col_from_frac(1.0, 1.0, 1.0)));
    let material_top = Rc::new(EmmissiveMaterial::new(col_from_frac(1.0, 1.0, 1.0)));


    /*let material_ground = Rc::new(DiffuseMaterial::new(col_from_frac(0.8, 0.8, 0.8)));
    let material_center = Rc::new(UVMaterial::new(Vector3d::new(235.0, 52.0, 192.0)));
    let material_left = Rc::new(RefractiveMaterial::new(col_from_frac(1.0, 1.0, 1.0), 1.5));
    let material_right = Rc::new(MetalMaterial::new(col_from_frac(0.8, 0.6, 0.2), 0.1));*/

    let scene_info = SceneInfo {
        sun_orientation: Vector3d::new(1.0, -1.0, 1.0),
        hittable_objects: //parse_obj_file("src/resources/teapot.obj", 1.0, Vector3d::default()),
        vec![
            parse_obj_file("src/resources/teapot.obj", 0.25, Vector3d::new(-0.05, 0.25, 0.0)),
            /*Box::new(Sphere::new(Vector3d::new(0.0, -1000.0, 0.0), 1000.0, Box::new(material_metal))),
            Box::new(Sphere::new(Vector3d::new(-1000.5, 0.0, 0.0), 1000.0, Box::new(material_left))),
            Box::new(Sphere::new(Vector3d::new(1000.5, 0.0, 0.0), 1000.0, Box::new(material_right))),
            Box::new(Sphere::new(Vector3d::new(0.0, 0.0, 1002.0), 1000.0, Box::new(material_back_front.clone()))),
            Box::new(Sphere::new(Vector3d::new(0.0, 1001.0, 0.0), 1000.0, Box::new(material_top))),
            Box::new(Sphere::new(Vector3d::new(0.0, 0.0, -1010.0), 1000.0, Box::new(material_back_front))),*/
            //Box::new(Sphere::new(Vector3d::new(0.0, 1.9, 0.0), 1.0, Box::new(material_back_front.clone()))),
        ]
    };

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
        println!();
        println!("estimated time left: {}s", (std::time::Instant::now() - start).as_secs_f64() / (pix_y as f64 + 1.0) * (data.canvas_height - pix_y) as f64);
        //println!("{}/{}", pix_y, data.canvas_height);
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

        /*canvas.set_draw_color(sdl2::pixels::Color::RGB(rng.gen(), rng.gen(), rng.gen()));
        canvas.clear();
        canvas.present();*/
    }
}
