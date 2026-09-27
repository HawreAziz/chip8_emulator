use sdl2::{event::Event, keyboard::Keycode, sys::KeyCode};

use crate::chip8_sdl::Sdl;

#[derive(PartialEq)]
pub enum State {
    RUNNING,
    PAUSED,
    QUIT,
}

fn map_key(keycode: Keycode) -> Option<usize> {
    match keycode {
        Keycode::NUM_1 => Some(0x1),
        Keycode::NUM_2 => Some(0x2),
        Keycode::NUM_3 => Some(0x3),
        Keycode::C => Some(0xC),
        Keycode::NUM_4 => Some(0x4),
        Keycode::NUM_5 => Some(0x5),
        Keycode::NUM_6 => Some(0x6),
        Keycode::D => Some(0xD),
        Keycode::NUM_7 => Some(0x7),
        Keycode::NUM_8 => Some(0x8),
        Keycode::NUM_9 => Some(0x9),
        Keycode::E => Some(0xE),
        Keycode::A => Some(0xA),
        Keycode::Num0 => Some(0x0),
        Keycode::B => Some(0xB),
        Keycode::F => Some(0xF),
        _ => None,
    }
}

pub fn input_handler(sdl: &mut Sdl, chip8: &mut chip8_core::Chip8) {
    for event in sdl.event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => {
                sdl.state = State::QUIT;
                break;
            }
            Event::KeyDown {
                keycode: Some(keycode),
                ..
            } => {
                if keycode == Keycode::Escape {
                    sdl.state = State::QUIT;
                    break;
                }
                if keycode == Keycode::Space {
                    sdl.state = if sdl.state == State::PAUSED {
                        State::RUNNING
                    } else {
                        State::PAUSED
                    };
                }
                if let Some(index) = map_key(keycode) {
                    println!("Pressed key: {:02x}", index);
                    chip8.set_key(index, true);
                }

                if keycode == Keycode::J {
                    let mut wave = sdl.audio_device.lock();
                    if wave.volume > 0.0 {
                        wave.volume -= 0.01;
                    }
                }
                if keycode == Keycode::P {
                    let mut wave = sdl.audio_device.lock();
                    if wave.volume < 1.0 {
                        wave.volume += 0.01;
                    }
                }

                if keycode == Keycode::UP {
                    if sdl.config.color_lerp_rate < 1.0 {
                        sdl.config.color_lerp_rate += 0.1;
                    }
                }

                if keycode == Keycode::DOWN {
                    if sdl.config.color_lerp_rate > 0.2 {
                        sdl.config.color_lerp_rate -= 0.1;
                    }
                }
            }
            Event::KeyUp {
                keycode: Some(keycode),
                ..
            } => {
                if let Some(index) = map_key(keycode) {
                    println!("Key {:02x} released", index);
                    chip8.set_key(index, false);
                }
            }

            _ => (),
        }
    }
}
