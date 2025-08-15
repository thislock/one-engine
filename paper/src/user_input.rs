use sdl3::keyboard::Keycode;

use crate::{
  maths::{self, Angle},
};

type InputFunction = Box<dyn Fn(&mut Vec<InputType>) -> ()>;
struct InputWrapper {
  keycode: Keycode,
  is_pressed: bool,
  action: InputFunction,
}

impl InputWrapper {
  pub fn new(key: Keycode, action: InputFunction) -> Self {
    Self {
      keycode: key,
      action,
      is_pressed: false,
    }
  }

  pub fn run_logic(&self, movement: &mut Vec<InputType>) {
    (self.action)(movement);
  }
}

#[derive(Debug)]
pub struct MovementDirection {
  pub direction: maths::Scalar,
}

#[derive(Debug)]
pub enum InputType {
  MoveCamera(MovementDirection),
  RotateCamera(MovementDirection),
}

pub struct MovementHandler {
  unread_movement: Vec<InputType>,
  input_wrappers: Vec<InputWrapper>,
}

fn add_scalar(input: &mut Vec<InputType>, rot_degrees: f64, magnitude: f64) {
  input.push(InputType::MoveCamera(MovementDirection {
    direction: maths::Scalar {
      magnitude,
      angle: Angle::from_degrees(rot_degrees),
    },
  }))
}

impl MovementHandler {
  pub fn new() -> Self {
    Self {
      unread_movement: vec![],
      input_wrappers: Self::get_input(),
    }
  }

  fn get_input() -> Vec<InputWrapper> {
    const CAMERA_SPEED: f64 = 1.0;
    return vec![
      InputWrapper::new(
        Keycode::W,
        Box::new(|movement| add_scalar(movement, 0.0, CAMERA_SPEED)),
      ),
      InputWrapper::new(
        Keycode::S,
        Box::new(|movement| add_scalar(movement, 180.0, CAMERA_SPEED)),
      ),
      InputWrapper::new(
        Keycode::A,
        Box::new(|movement| add_scalar(movement, -90.0, CAMERA_SPEED)),
      ),
      InputWrapper::new(
        Keycode::D,
        Box::new(|movement| add_scalar(movement, 90.0, CAMERA_SPEED)),
      ),
    ];
  }

  pub fn poll_movement(&mut self, event: &sdl3::event::Event) {

    self.unread_movement.clear();

    match event {
      sdl3::event::Event::KeyDown { keycode, .. } => {
        if let Some(key) = keycode {
          self.set_keys(key, true);
        }
      }
      sdl3::event::Event::KeyUp { keycode, .. } => {
        if let Some(key) = keycode {
          self.set_keys(key, false);
        }
      }

      sdl3::event::Event::MouseMotion { xrel, yrel, .. } => {
        
        let x = *xrel;
        let y = *yrel;

        let magnitude = (x.powf(2.0) + y.powf(2.0)).sqrt() as f64;
        let rot = maths::Angle::from_radians((y.atan2(x)) as f64);
        
        let scalar = maths::Scalar::new(magnitude, rot);

        self
          .unread_movement
          .push(InputType::RotateCamera(MovementDirection {
            direction: scalar,
          }));
      }

      _ => {}
    }

    for wrapper in self.input_wrappers.iter() {
      if wrapper.is_pressed {
        wrapper.run_logic(&mut self.unread_movement);
      }
    }
  }

  fn set_keys(&mut self, key: &Keycode, pressed: bool) {
    for wrapper in &mut self.input_wrappers {
      if wrapper.keycode == *key {
        wrapper.is_pressed = pressed;
      }
    }
  }

  pub fn get_movement<'a>(&'a self) -> Vec<&'a InputType> {
    return self.unread_movement.iter().collect();
  }
}
