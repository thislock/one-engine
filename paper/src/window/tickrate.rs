use std::{
  thread, time::{self, Instant}
};

pub struct TimeMeasurer {
  last_time: Instant,
  now: Instant,
  average_time: Vec<f32>,
  average_before_div: f64,
  average_float: f64,
} 

impl TimeMeasurer {

  pub fn new() -> Self {
    let now = Instant::now();
    Self {
      last_time: now, now,
      average_time: vec![],
      average_before_div: 0.0,
      average_float: 0.0,
    }
  }

  pub fn start_measure(&mut self) {
    self.last_time = time::Instant::now();
  }

  pub fn stop_measure(&mut self) -> f64 {
    self.now = time::Instant::now();
    let dur = self.now.duration_since(self.last_time);
    self.add_timestamp(dur);
    return dur.as_secs_f64();
  }

  pub fn get_average(&self) -> f64 { return self.average_float }


  fn snip_average(&mut self) {
    let half_point = self.average_time.len() / 2;
    let _dropped_half = self.average_time.split_off(half_point);
    self.recalculate_average();
  }

  fn recalculate_average(&mut self) {
    self.average_before_div = 0.0;
    for element in &self.average_time {
      self.average_before_div += *element as f64;
    }
  }

  fn set_average(&mut self) {
    if self.average_before_div != 0.0 && self.average_time.len() != 0 {
      self.average_float = self.average_before_div / self.average_time.len() as f64
    } else {
      self.average_float = 0.0;
    }
  }

  fn add_timestamp(&mut self, dur: time::Duration) {
    const CAPACITY: usize = 100;
    
    if self.average_time.len() > CAPACITY {
      self.snip_average();
    }

    let dur_f = dur.as_secs_f64();
    self.average_before_div += dur_f;
    self.set_average();

    self.average_time.push(dur_f as f32);

  }

}

#[allow(unused)]
pub struct Tickrate {
  time_measuring: TimeMeasurer,
  delta_time: f64,
  wait_time: f64,
  target_framerate: u16,
}

impl Tickrate {
  pub fn new() -> Self {
    Self {
      time_measuring: TimeMeasurer::new(),
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
    if self.wait_time > 0.0 {
      thread::sleep(time::Duration::from_secs_f64(self.wait_time));
    }
  }

  /// will record the time it takes for the frame to run
  /// ONLY ONE ONCE PER FRAME AT THE VERY END OF CALCULATIONS, then sleep
  pub fn tick(&mut self) {
    let render_time = self.time_measuring.stop_measure();
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

    self.time_measuring.start_measure();
  }
}
