mod keypad_handler;
mod sdl;

fn main() {
    let mut window = sdl::Sdl::new();
    window.run();
    window.cleanup();
}
