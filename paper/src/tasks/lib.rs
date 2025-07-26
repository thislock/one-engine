
use crate::{engine::Engine, tasks::Task};


mod queue_render;

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
struct BurnerTask {
}

#[allow(unused)]
fn pointlessly_complex_task(x: i32) -> i32 {
    
  let mut x2 = x;

  for _ in 0..100 {
    x2 += 1;
  }

  x + x2
}

impl Task for BurnerTask {
  fn get_type(&self) -> crate::tasks::TaskType {
    crate::tasks::TaskType::Looping
  }
  fn run_task(
      &mut self,
      _message: &mut crate::tasks::TaskMessenger,
      // the time since the function was ran last
      delta_time: f32,
    ) -> anyhow::Result<()> {
    
    for i in 0..100_000 {
      print!("{}", pointlessly_complex_task((i as f32 * delta_time) as i32));
    }
    Ok(())
  }
}


pub fn init_tasks(engine: &mut Engine) {
  engine.task_service.add_tasks(
    vec![
      Box::new(queue_render::QueueRender {window: engine.get_window()}),
    ]
  );

    // stress test

    //let mut stress: Vec<Box<(dyn Task + Send + 'static)>> = vec![];
    //for _ in 0..300 {stress.push(Box::new(BurnerTask {}));}
    //engine.task_service.add_tasks(stress);
}