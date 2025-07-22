use std::{sync::Arc, thread, time::{Duration, Instant}};

use winit::window::Window;


pub struct Tickrate {
    target_framerate: u32,
    last_tick: Instant,
    window: Arc<Window>,
    delta_time: f64,

    #[allow(unused)]
    start_time: Instant,
}


fn get_refresh_rate(window: Arc<Window>) -> u32 {
    // figure out what refresh rate the screen is,
    // otherwise just assume it's 60 hertz
    const FALLBACK_FPS: u32 = 60;
    if let Some(monitor) = window.current_monitor() {
        if let Some(hertz_millis) = monitor.refresh_rate_millihertz() {
            return hertz_millis / 1000;
        } 
    }
    // else
    return FALLBACK_FPS
}

impl Tickrate {
    pub fn new(window: Arc<Window>) -> Self {
        Self {
            target_framerate: get_refresh_rate(window.clone()),
            last_tick: Instant::now(),
            start_time: Instant::now(),
            window,
            delta_time: 0.0,
        }
    }

    pub fn get_delta(&self) -> f32 {
        self.delta_time as f32
    }

    // will sleep inbetween frames
    pub fn tick_sleep(&mut self) {
        let render_time = Instant::now().duration_since(self.last_tick).as_secs_f64();
        let frame_time = 1.0/(self.target_framerate as f64);
        
        let wait_time = frame_time - render_time;
        
        if wait_time > 0.0 {
            self.delta_time = frame_time;
            thread::sleep(Duration::from_secs_f64(wait_time));
        } else {
            self.delta_time = render_time;
        }
        
        self.last_tick = Instant::now();
        
        // might become a performance issue later idk
        self.target_framerate = get_refresh_rate(self.window.clone());
    }
}