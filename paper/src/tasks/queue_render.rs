use std::sync::Arc;
use crate::tasks::{LoopGroup, Task, TaskMessenger, TaskType};

pub struct QueueRender {
  pub window: Arc<winit::window::Window>,
  pub loop_group: LoopGroup,
}

impl Task for QueueRender {
  fn get_type(&self) -> crate::tasks::TaskType {
    TaskType::Looping(self.loop_group.clone())
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