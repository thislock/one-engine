use std::sync::Arc;
use crate::tasks::{Task, TaskType, TaskMessenger};

pub struct QueueRender {
  pub window: Arc<winit::window::Window>,
}

impl Task for QueueRender {
  fn get_type(&self) -> crate::tasks::TaskType {
    TaskType::Looping
  }
  fn run_task(
      &mut self,
      _messages: &mut TaskMessenger,
      // the time since the function was ran last
      _delta_time: f32,
    ) -> anyhow::Result<()> 
  {        
    self.window.request_redraw();
    Ok(())
  }
}