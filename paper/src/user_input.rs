use std::sync::Arc;

use sdl3::keyboard::Keycode;

use crate::{
  engine::Engine,
  maths::{self, Angle},
  SdlHandle,
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

#[derive(Debug, Clone, Copy)]
pub struct MovementDirection {
  pub direction: maths::Scalar,
}

#[derive(Debug, Clone, Copy)]
pub struct RotationDirection {
  pub pitch: f64, 
  pub yaw: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum InputType {
  MoveCamera(MovementDirection),
  RotateCamera(RotationDirection),
}

pub struct MovementHandler {
  input_wrappers: Vec<InputWrapper>,
  mouse_util: sdl3::mouse::MouseUtil,
  window: Arc<sdl3::video::Window>,
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
  pub fn new(sdl_context: &SdlHandle, window: Arc<sdl3::video::Window>) -> Self {
    Self {
      input_wrappers: Self::get_input(),
      mouse_util: sdl3::Sdl::mouse(&sdl_context.sdl_context),
      window,
    }
  }

  fn get_input() -> Vec<InputWrapper> {
    const CAMERA_SPEED: f64 = 2.0;
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

  pub fn poll_movement(
    engine: &mut Engine,
    unread_movement: &mut Vec<InputType>,
    event: &sdl3::event::Event,
  ) {
    match event {
      sdl3::event::Event::KeyDown { keycode, .. } => {
        if let Some(key) = keycode {
          engine.user_input.set_keys(key, true);
        }
      }
      sdl3::event::Event::KeyUp { keycode, .. } => {
        if let Some(key) = keycode {
          engine.user_input.set_keys(key, false);
        }
      }

      sdl3::event::Event::MouseMotion { x, y, .. } => {
        engine
          .user_input
          .calculate_mouse_delta(unread_movement, *x, *y);
      }

      _ => {}
    }

  }

  pub fn apply_movement(engine: &mut Engine, unread_movement: &mut Vec<InputType>) {
    // loop through all the movement handlers and run them if they are active
    for wrapper in engine.user_input.input_wrappers.iter() {
      if wrapper.is_pressed {
        wrapper.run_logic(unread_movement);
      }
    }

    engine
      .camera
      .update_camera(&unread_movement, engine.tickrate.get_delta());
  }

  fn calculate_mouse_delta(&mut self, unread_movement: &mut Vec<InputType>, x: f32, y: f32) {
    let window_size = self.window.size();
    let reset_pos = (window_size.0 / 2, window_size.1 / 2);
    let sensitivity = 2.0;

    let x = (x - reset_pos.0 as f32) as f64 * sensitivity;
    let y = (y - reset_pos.1 as f32) as f64 * sensitivity;

    let rot_dir = InputType::RotateCamera(
      RotationDirection { pitch: x, yaw: y }
    );

    unread_movement.push(rot_dir);

    self
      .mouse_util
      .warp_mouse_in_window(&self.window, reset_pos.0 as f32, reset_pos.1 as f32);
  }

  fn set_keys(&mut self, key: &Keycode, pressed: bool) {
    for wrapper in &mut self.input_wrappers {
      // check if its the correct key
      if wrapper.keycode == *key {
        wrapper.is_pressed = pressed;
      }

    }
  }
}
