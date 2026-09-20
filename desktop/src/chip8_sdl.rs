use crate::keypad_handler;
use chip8_core;
use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window};

pub struct Sdl {
    canvas: Canvas<Window>,
    pub event_pump: sdl2::EventPump, // Sparas här så att de inte försvinner ur minnet
    _sdl_context: sdl2::Sdl,         // Håller SDL vid liv under hela structens li
    pub state: keypad_handler::State,
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

    pub fn draw_screen(&mut self, chip8: &chip8_core::Chip8, config: &chip8_core::Config) {
        self.canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        self.canvas.clear();
        let fg_r: u8 = ((config.fg_color >> 24) & 0xFF) as u8;
        let fg_g: u8 = ((config.fg_color >> 16) & 0xFF) as u8;
        let fg_b: u8 = ((config.fg_color >> 8) & 0xFF) as u8;
        let fg_a: u8 = (config.fg_color & 0xFF) as u8;

        self.canvas
            .set_draw_color(Color::RGBA(fg_r, fg_g, fg_b, fg_a));
        for (i, pixel) in chip8.get_display().iter().enumerate() {
            if *pixel {
                let x = (i % config.screen_width) as u32;
                let y = (i / config.screen_width) as u32;

                let rect = Rect::new(
                    (x * config.scale) as i32,
                    (y * config.scale) as i32,
                    config.scale,
                    config.scale,
                );
                self.canvas.fill_rect(rect).unwrap();
            }
        }
        self.canvas.present();
    }
}
