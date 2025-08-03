use crate::{paper_error, tasks::{LoopGroup, Task, TaskMessenger, TaskType}};
use std::sync::Arc;

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
    messages: &mut TaskMessenger,
    // the time since the function was ran last
    _delta_time: f32,
  ) -> anyhow::Result<()> {
    
    let mut do_redraw = true;
    let msg = messages.reciever.try_recv();

    if let Err(closed) = msg {
      match closed {
        std::sync::mpsc::TryRecvError::Disconnected => {
          let _ = messages.self_sender.send(crate::tasks::ToTask::Exit);
          do_redraw = false;
        },
        _ => do_redraw = true,
      }
    }

    if do_redraw {
      self.window.request_redraw()
    }
    
    Ok(())
  }
}
