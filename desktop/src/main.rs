use chip8_core;
mod chip8_sdl;
mod keypad_handler;
use sdl2::log::log_error;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        log_error("Wrong number of argument usage: Cargo run <file-name>");
        return;
    }
    let config = chip8_core::Config::new();
    let mut chip8 = chip8_core::Chip8::new(&args[1]);
    let mut window = chip8_sdl::Sdl::new();

    loop {
        keypad_handler::input_handler(&mut window.event_pump, &mut window.state);
        match window.state {
            keypad_handler::State::PAUSED => continue,
            keypad_handler::State::QUIT => break,
            _ => (),
        }
        // for _ in 0..TICKS_PER_FRAME {
        //     cpu.tick();
        // }
        chip8.execute();
        window.draw_screen(&chip8, &config);
    }

    window.cleanup();
}
