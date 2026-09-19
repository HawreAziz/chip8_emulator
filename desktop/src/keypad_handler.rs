use sdl2::{event::Event, keyboard::Keycode};

#[derive(PartialEq)]
pub enum State {
    RUNNING,
    PAUSED,
    QUIT,
}
pub fn input_handler(event_pump: &mut sdl2::EventPump, state: &mut State) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => *state = State::QUIT,
            Event::KeyDown {
                keycode: Some(keycode),
                ..
            } => {
                if keycode == Keycode::Escape {
                    *state = State::QUIT;
                    break;
                }
                if keycode == Keycode::Space {
                    *state = if *state == State::PAUSED {
                        State::RUNNING
                    } else {
                        println!("paused");
                        State::PAUSED
                    };
                }
                // TODO match the other keycodes here
            }

            _ => (),
        }
    }
}
