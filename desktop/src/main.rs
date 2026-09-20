use chip8_core;
mod keypad_handler;
mod sdl;

fn main() {
    let chip8 = chip8_core::Chip8::new();
    println!("{:#?}", chip8);
    let mut window = sdl::Sdl::new();
    window.run();
    window.cleanup();
}
