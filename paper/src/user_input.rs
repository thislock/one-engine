use sdl3::{keyboard::Keycode, sys::events::SDL_EVENT_KEY_DOWN};

use crate::{
  maths::{self, Angle},
  SdlHandle,
};

pub struct MovementDirection {
  direction: maths::Scalar,
}

pub enum InputType {
  MoveCamera(MovementDirection),
  RotateCamera(MovementDirection),
}

pub struct MovementHandler {
  unread_movement: Vec<InputType>,
}

impl MovementHandler {
  pub fn new() -> Self {
    Self {
      unread_movement: vec![],
    }
  }

  pub fn poll_movement(&mut self, event: &sdl3::event::Event) {
    match event {
      sdl3::event::Event::KeyUp { keycode, .. } => {
        if let Some(key) = keycode {
          match key {
            Keycode::W => self
              .unread_movement
              .push(InputType::MoveCamera(MovementDirection {
                direction: maths::Scalar {
                  magnitude: 1.0,
                  angle: Angle::from_degrees(0.0),
                },
              })),
            _ => {}
          }
        }
      }

      _ => {}
    }
  }
}
