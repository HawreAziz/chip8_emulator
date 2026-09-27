use crate::chip8_audio::SquareWave;
use crate::keypad_handler;
use chip8_core;
use sdl2::audio::{AudioDevice, AudioSpecDesired};
use sdl2::{TimerSubsystem, pixels::Color, rect::Rect, render::Canvas, video::Window};

pub struct Config {
    bg_color: u32,
    fg_color: u32,
    screen_width: usize,
    screen_height: usize,
    scale: u32,
    pixel_color: [u32; chip8_core::SCREEN_WIDTH * chip8_core::SCREEN_HEIGHT],
    pub color_lerp_rate: f32,
}

impl Config {
    pub fn new(scale_factor: u32) -> Config {
        Config {
            bg_color: 0x000000FF,
            fg_color: 0xFFFFFFFF,
            screen_width: chip8_core::SCREEN_WIDTH,
            screen_height: chip8_core::SCREEN_HEIGHT,
            pixel_color: [0; chip8_core::SCREEN_WIDTH as usize
                * chip8_core::SCREEN_HEIGHT as usize],
            scale: scale_factor,
            color_lerp_rate: 0.1,
        }
    }
}

pub struct Sdl {
    canvas: Canvas<Window>,
    pub event_pump: sdl2::EventPump, // Sparas här så att de inte försvinner ur minnet
    _sdl_context: sdl2::Sdl,         // Håller SDL vid liv under hela structens li
    pub state: keypad_handler::State,
    pub sdl_timer: TimerSubsystem,
    pub audio_device: AudioDevice<SquareWave>,
    pub config: Config,
}

impl Sdl {
    pub fn new(scale_factor: i32) -> Sdl {
        let sdl_context = sdl2::init().unwrap();
        let video_sybsystem = sdl_context.video().unwrap();
        let sdl_timer = sdl_context.timer().unwrap();
        let scale = if scale_factor > 0 {
            scale_factor as u32
        } else {
            15
        };
        let config = Config::new(scale);
        let window = video_sybsystem
            .window(
                "Chip8-emulator",
                config.screen_width as u32 * scale,
                config.screen_height as u32 * scale,
            )
            .position_centered()
            .opengl()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().present_vsync().build().unwrap();
        canvas.clear();
        canvas.present();
        let event_pump = sdl_context.event_pump().unwrap();

        let audio_subsystem = sdl_context.audio().unwrap();
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };
        let audio_device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                let target_freq = 440.0;
                SquareWave::new(target_freq / spec.freq as f32, 0.0, 0.1)
            })
            .unwrap();

        Sdl {
            canvas: canvas,
            event_pump: event_pump,
            _sdl_context: sdl_context,
            state: keypad_handler::State::RUNNING,
            sdl_timer,
            audio_device,
            config,
        }
    }

    pub fn update_timer(&self, chip8: &mut chip8_core::Chip8) {
        chip8.tick_delay();
        if chip8.get_sound_timer() > 0 {
            self.audio_device.resume();
        } else {
            self.audio_device.pause();
        }
    }

    pub fn cleanup(self) {
        drop(self.canvas);
    }

    fn as_rgba(&self, color: u32) -> (u8, u8, u8, u8) {
        let r = ((color >> 24) & 0xFF) as u8;
        let g = ((color >> 16) & 0xFF) as u8;
        let b = ((color >> 8) & 0xFF) as u8;
        let a = (color & 0xFF) as u8;
        return (r, g, b, a);
    }

    fn color_lerp(&mut self, index: usize, target_color: u32) {
        let (s_r, s_g, s_b, s_a) = self.as_rgba(self.config.pixel_color[index]);
        let (e_r, e_g, e_b, e_a) = self.as_rgba(target_color);

        let rate = self.config.color_lerp_rate as f32;
        let ret_r = (s_r as f32 + (e_r as f32 - s_r as f32) * rate) as u8;
        let ret_g = (s_g as f32 + (e_g as f32 - s_g as f32) * rate) as u8;
        let ret_b = (s_b as f32 + (e_b as f32 - s_b as f32) * rate) as u8;
        let ret_a = (s_a as f32 + (e_a as f32 - s_a as f32) * rate) as u8;
        self.config.pixel_color[index] = ((ret_r as u32) << 24)
            | ((ret_g as u32) << 16)
            | ((ret_b as u32) << 8)
            | (ret_a as u32);
    }

    pub fn draw_screen(&mut self, chip8: &chip8_core::Chip8) {
        let bg_r: u8 = ((self.config.bg_color >> 24) & 0xFF) as u8;
        let bg_g: u8 = ((self.config.bg_color >> 16) & 0xFF) as u8;
        let bg_b: u8 = ((self.config.bg_color >> 8) & 0xFF) as u8;
        let bg_a: u8 = (self.config.bg_color & 0xFF) as u8;
        self.canvas
            .set_draw_color(Color::RGBA(bg_r, bg_g, bg_b, bg_a));
        self.canvas.clear();

        for (i, pixel) in chip8.get_display().iter().enumerate() {
            let target_color = if *pixel {
                self.config.fg_color
            } else {
                self.config.bg_color
            };
            if self.config.pixel_color[i] != target_color {
                self.color_lerp(i, target_color);
            }
            if *pixel {
                let x = (i % self.config.screen_width) as u32;
                let y = (i / self.config.screen_width) as u32;

                let rect = Rect::new(
                    (x * self.config.scale) as i32,
                    (y * self.config.scale) as i32,
                    self.config.scale,
                    self.config.scale,
                );
                let fg_r: u8 = ((self.config.pixel_color[i] >> 24) & 0xFF) as u8;
                let fg_g: u8 = ((self.config.pixel_color[i] >> 16) & 0xFF) as u8;
                let fg_b: u8 = ((self.config.pixel_color[i] >> 8) & 0xFF) as u8;
                let fg_a: u8 = (self.config.pixel_color[i] & 0xFF) as u8;
                self.canvas
                    .set_draw_color(Color::RGBA(fg_r, fg_g, fg_b, fg_a));
                self.canvas.fill_rect(rect).unwrap();
            }
        }
        self.canvas.present();
    }
}
