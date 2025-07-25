use std::{sync::Arc, time::{Duration, Instant}};
use crate::tasks::{self, Task, TaskType, TaskMessenger};

pub struct QueueRender {
    pub window: Arc<winit::window::Window>,
}

impl Task for QueueRender {
    fn get_type(&self) -> crate::tasks::TaskType {
        TaskType::LOOPING
    }
    fn run_task(
            &mut self,
            messages: &mut TaskMessenger,
            // the time since the function was ran last
            _delta_time: f32,
        ) -> anyhow::Result<()> 
    {        
        self.window.request_redraw();

        // TODO: vsync
        let framerate: f32 = 60.0;
        let next = Instant::now().checked_add(Duration::from_secs_f32(1.0/framerate)).unwrap_or(Instant::now());
        messages.self_sender.send(tasks::ToTask::Schedule(next))?;

        Ok(())
    }
}