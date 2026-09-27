use chip8_core;
mod chip8_audio;
mod chip8_sdl;
mod keypad_handler;
use std::{env, time::Duration};

const TICKS_PER_FRAME: usize = 8;

struct Args {
    rom_file: String,
    scale_factor: i32,
}

fn get_args() -> Args {
    let mut arg_obj = Args {
        rom_file: "".to_string(),
        scale_factor: -1,
    };
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--rom-file" || arg == "-r" {
            let arg_value = args.next().expect("--rom-file/-r has no value");
            arg_obj.rom_file = arg_value;
        } else if arg == "--scale-factor" || arg == "-s" {
            let arg_value = args.next().expect("--scale/-s has no value");
            arg_obj.scale_factor = arg_value.parse().expect("Wrong value for scale_factor");
        }
    }
    if arg_obj.rom_file.len() == 0 {
        panic!("--rom-file/-r is a required flag");
    }
    arg_obj
}

fn main() {
    let args = get_args();
    let mut chip8 = chip8_core::Chip8::new(args.rom_file);
    let mut window = chip8_sdl::Sdl::new(args.scale_factor);

    loop {
        keypad_handler::input_handler(&mut window, &mut chip8);
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
        if chip8.get_draw() {
            window.draw_screen(&chip8);
            chip8.set_draw(false);
        }
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
