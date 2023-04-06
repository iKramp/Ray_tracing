extern crate sdl2;
extern crate rand;

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;


pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("pc sim", WIDTH * 2, HEIGHT * 2)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();//code above inits sdl2 somehow, idk what it does

    loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => return,
                _ => {}
            }
        }
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();
    }
}