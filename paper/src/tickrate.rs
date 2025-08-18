use std::{thread, time::{self, Instant}};

#[allow(unused)]
pub struct Tickrate {
  last_tick: Instant,
  delta_time: f64,
  wait_time: f64,

  start_time: Instant,
  target_framerate: u16,
}

impl Tickrate {
  pub fn new() -> Self {
    Self {
      last_tick: Instant::now(),
      start_time: Instant::now(),
      delta_time: 0.0,
      target_framerate: 60,
      wait_time: 0.0,
    }
  }

  pub fn get_delta(&self) -> f32 {
    self.delta_time as f32
  }

  fn target_frame_time(&self) -> f64 {
    1.0 / self.target_framerate as f64
  }

  pub fn sleep_until_next_frame(&self) {
    if self.wait_time == 0.0 {
      thread::sleep(time::Duration::from_secs_f64(self.wait_time));
    }
  }

  /// will record the time it takes for the frame to run
  /// ONLY ONE ONCE PER FRAME AT THE VERY END OF CALCULATIONS, then sleep
  pub fn tick(&mut self) {

    let render_time = Instant::now().duration_since(self.last_tick).as_secs_f64();
    // if rendering is shorter than the target framerate
    if render_time < self.target_frame_time() {
      self.wait_time = self.target_frame_time() - render_time;
      self.delta_time = self.target_frame_time();
    } 
    // otherwise, the frame took too long to compute, set it directly, and skip waiting at the end of the frame.
    else {
      self.delta_time = render_time;
      self.wait_time = 0.0;
    }
    self.last_tick = Instant::now();
  }
}
