use std::{sync::Arc, time::{Duration, Instant}};
use crate::tasks::{self, Task, TaskType, TaskMessenger};

pub struct QueueRender {
    pub window: Arc<winit::window::Window>,
}

impl Task for QueueRender {
    fn get_importance(&self) -> crate::tasks::TaskType {
        TaskType::LOOPING
    }
    fn run_task(
            &mut self,
            messages: &mut TaskMessenger,
            // the time since the function was ran last
            delta_time: f32,
        ) -> anyhow::Result<()> 
    {
        let _ = delta_time;
        
        self.window.request_redraw();
        
        messages.self_sender.send(tasks::ToTask::Schedule(Instant::now().checked_add(Duration::from_secs_f32(1.0/60.0)).unwrap()))?;

        Ok(())
    }
}