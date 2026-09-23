use chip8_core;
mod chip8_sdl;
mod keypad_handler;
use sdl2::log::log_error;
use std::{env, time::Duration};

const TICKS_PER_FRAME: usize = 10;

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
        keypad_handler::input_handler(&mut window.event_pump, &mut window.state, &mut chip8);
        match window.state {
            keypad_handler::State::PAUSED => continue,
            keypad_handler::State::QUIT => break,
            _ => (),
        }

        let start_time = window.sdl_timer.performance_counter();
        for _ in 0..TICKS_PER_FRAME {
            chip8.execute();
        }
        let end_time = window.sdl_timer.performance_counter();
        let time_elapsed = ((end_time - start_time) * 1000) as f64
            / window.sdl_timer.performance_frequency() as f64;
        window.draw_screen(&chip8, &config);
        window.update_timer(&mut chip8);
        let elapsed = if 16.7 > time_elapsed {
            16.7 as f64 - time_elapsed
        } else {
            0.0
        };
        std::thread::sleep(Duration::from_millis(elapsed as u64));
    }

    window.cleanup();
}
