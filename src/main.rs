pub mod ray;
use rand::prelude::*;
use ray::{data::*, material::*, trace::*};
use std::rc::Rc;
use vector3d::Vector3d;
use crate::ray::get_canvas_and_pump;
use crate::ray::RayReturnState::Ray;

const SHADER: &[u8] = include_bytes!(env!("shader.spv"));



pub fn main() {
    let data = CamData::default();

    let image = image::open("src/resources/earth_4.jpg").unwrap();
    let resources = Rc::new(Resources { earth: image });
    let _normal_material = Rc::new(NormalMaterial {});
    let scene_info = SceneInfo::default();

    let (mut canvas , mut event_pump) = get_canvas_and_pump(data.canvas_width as u32, data.canvas_height as u32);

    println!("rendering...");

    if ray::Ray::render(&mut canvas, &mut event_pump, &data, &scene_info, resources) {
        return;
    }











    println!("done rendering");

    canvas.present();

    loop {
        for event in event_pump.poll_iter() {
            if let sdl2::event::Event::Quit { .. } = event {
                return;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100))
    }
}
