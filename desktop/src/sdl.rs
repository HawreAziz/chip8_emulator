use crate::keypad_handler;
use chip8_core;
use sdl2::{event::Event, render::Canvas, video::Window};

pub struct Sdl {
    canvas: Canvas<Window>,
    event_pump: sdl2::EventPump, // Sparas här så att de inte försvinner ur minnet
    _sdl_context: sdl2::Sdl,     // Håller SDL vid liv under hela structens li
    state: keypad_handler::State,
}

impl Sdl {
    pub fn new() -> Sdl {
        let sdl_context = sdl2::init().unwrap();
        let video_sybsystem = sdl_context.video().unwrap();
        let window = video_sybsystem
            .window(
                "Chip8-emulator",
                chip8_core::WINDOW_WIDTH,
                chip8_core::WINDOW_HEIGHT,
            )
            .position_centered()
            .opengl()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().present_vsync().build().unwrap();
        canvas.clear();
        canvas.present();
        let event_pump = sdl_context.event_pump().unwrap();
        Sdl {
            canvas: canvas,
            event_pump: event_pump,
            _sdl_context: sdl_context,
            state: keypad_handler::State::RUNNING,
        }
    }

    pub fn cleanup(self) {
        drop(self.canvas);
    }

    pub fn run(&mut self) {
        loop {
            // for evt in self.event_pump.poll_iter() {
            //     match evt {
            //         Event::Quit { .. } => {
            //             break 'gameloop;
            //         }
            //         _ => (),
            //     }
            // }
            keypad_handler::input_handler(&mut self.event_pump, &mut self.state);
            match self.state {
                keypad_handler::State::PAUSED => continue,
                keypad_handler::State::QUIT => break,
                _ => (),
            }
            // for _ in 0..TICKS_PER_FRAME {
            //     cpu.tick();
            // }
            // cpu.tick_timers();
            // draw_screen(&cpu, &mut canvas);
            println!("drawing");
            self.draw_screen();
        }
    }

    fn draw_screen(&mut self) {}
}
