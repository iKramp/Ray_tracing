pub mod data;
pub mod hit;
pub mod material;
pub mod trace;

use sdl2::render::WindowCanvas;
pub use material::*;
pub use trace::*;

pub fn get_canvas_and_pump(w: u32, h: u32) -> (WindowCanvas, sdl2::EventPump) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "modules tracer",
            w,
            h,
        )
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap(); //code above inits sdl2 somehow, idk what it does
    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    (canvas, event_pump)
}
