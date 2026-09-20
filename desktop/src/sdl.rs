use crate::keypad_handler;
use chip8_core;
use sdl2::{event::Event, pixels::Color, rect::Rect, render::Canvas, video::Window};

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
            self.draw_screen();
        }
    }

    fn draw_screen(&mut self) {
        // TODO screen should be updated with the backgroun and foreground colors comming from chip8
        // for now keep this
        let background: u32 = 0x000000FF;
        let foreground: u32 = 0xFFFFFFFF;

        let bg_r: u8 = ((background >> 24) & 0xFF) as u8;
        let bg_g: u8 = ((background >> 16) & 0xFF) as u8;
        let bg_b: u8 = ((background >> 8) & 0xFF) as u8;
        let bg_a: u8 = (background & 0xFF) as u8;

        let fg_r: u8 = ((foreground >> 24) & 0xFF) as u8;
        let fg_g: u8 = ((foreground >> 16) & 0xFF) as u8;
        let fg_b: u8 = ((foreground >> 8) & 0xFF) as u8;
        let fg_a: u8 = (foreground & 0xFF) as u8;

        // clear screen
        self.canvas
            .set_draw_color(Color::RGBA(bg_r, bg_g, bg_b, bg_a));
        self.canvas.clear();

        self.canvas
            .set_draw_color(Color::RGBA(fg_r, fg_g, fg_b, fg_a));

        let rect = Rect::new(50, 50, 50, 50);

        self.canvas.fill_rect(rect).unwrap();
        self.canvas.present();
    }
}
