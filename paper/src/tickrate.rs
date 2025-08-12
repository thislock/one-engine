use std::time::{self, Instant};

pub struct Tickrate {
  last_tick: Instant,
  delta_time: f64,

  #[allow(unused)]
  start_time: Instant,
}

impl Tickrate {
  pub fn new() -> Self {
    Self {
      last_tick: Instant::now(),
      start_time: Instant::now(),
      delta_time: 0.0,
    }
  }

  pub fn get_delta(&self) -> f32 {
    self.delta_time as f32
  }

  pub fn get_sleep_time(&self) -> time::Duration {
    time::Duration::from_secs_f32(1.0/60.0)
  }

  // will sleep inbetween frames
  pub fn tick(&mut self) {
    let render_time = Instant::now().duration_since(self.last_tick).as_secs_f64();
    self.delta_time = render_time;
    self.last_tick = Instant::now();
  }
}
