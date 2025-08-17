use std::sync::Arc;

use sdl3::keyboard::Keycode;

use crate::{
  engine::Engine, maths::{self, Angle}, SdlHandle
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

  pub fn poll_movement(unread_movement: &mut Vec<InputType>, engine: &mut Engine, event: &sdl3::event::Event) {

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
        engine.user_input.calculate_mouse_delta(unread_movement, *x, *y);
      }

      _ => {}
    }

    for wrapper in engine.user_input.input_wrappers.iter() {
      if wrapper.is_pressed {
        wrapper.run_logic(unread_movement);
      }
    }

  }

  pub fn apply_movement(engine: &mut Engine, unread_movement: &mut Vec<InputType>) {
    
    let mut move_cam = (0.0, 0.0);
    let mut rot_cam = (0.0, 0.0);
    
    for unread in unread_movement {
      match unread {
        InputType::MoveCamera(dir) => {
          move_cam.0 += dir.direction.magnitude;
          move_cam.1 += dir.direction.angle.as_radians();
        }
        InputType::RotateCamera(dir) => {
          rot_cam.0 += dir.direction.magnitude;
          rot_cam.1 += dir.direction.angle.as_radians();
        }
      }
    }

    let move_cam = InputType::MoveCamera(   MovementDirection {direction: maths::Scalar::new(move_cam.0, maths::Angle::from_radians(move_cam.1))});
    let rot_cam =  InputType::RotateCamera( MovementDirection {direction: maths::Scalar::new(rot_cam.0, maths::Angle::from_radians(rot_cam.1))});
    let movement = vec![move_cam, rot_cam];

    engine.camera.update_camera(movement, engine.tickrate.get_delta());
  }

  pub fn calculate_mouse_delta(&mut self, unread_movement: &mut Vec<InputType>, x: f32, y: f32) {
    let window_size = self.window.size();
    let reset_pos = (window_size.0 / 2, window_size.1 / 2);
    let sensitivity = 0.5;
  
    let x = (x - reset_pos.0 as f32) as f64 * sensitivity;
    let y = (y - reset_pos.1 as f32) as f64 * sensitivity;

    let magnitude = (x*x + y*y).sqrt() as f64;
    let rot = maths::Angle::from_radians((y.atan2(x)) as f64);

    let scalar = maths::Scalar::new(magnitude, rot);

    unread_movement
      .push(InputType::RotateCamera(MovementDirection {
        direction: scalar,
      }));

    self
      .mouse_util
      .warp_mouse_in_window(&self.window, reset_pos.0 as f32, reset_pos.1 as f32);
  }

  fn set_keys(&mut self, key: &Keycode, pressed: bool) {
    for wrapper in &mut self.input_wrappers {
      if wrapper.keycode == *key {
        wrapper.is_pressed = pressed;
      }
    }
  }

}
