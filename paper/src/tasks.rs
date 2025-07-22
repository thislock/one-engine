// a system for scheduling tasks of varying importance,
// with a delta so less important tasks can be ignored to avoid lag spikes

use std::{collections::HashMap, time::{Duration, Instant}};

pub trait Task {
    fn get_importance(&self) -> TaskImportance;    
    fn run_task(
        &mut self,
        // the time since the function was ran last
        delta_time: f32,
    ) -> anyhow::Result<()>;
}

use strum::IntoEnumIterator;
use strum_macros::EnumIter; 

#[derive(Debug, EnumIter)]
pub enum TaskImportance {
    // essential to the program, run at the cost of framerate
    CRITICAL,
    // will only cause minor lag, but will stop after lagging a little bit
    MEDIUM,
    // run only if there is spare processing
    MINIMAL,
}

struct TimeInfo {
    running_time_history: [Option<std::time::Duration>; 128],
    history_tracker: usize,

    avr_time_ran: std::time::Duration,

    last_time_ran: std::time::Instant,
}

impl TimeInfo {
    fn new() -> Self {
        Self {
            running_time_history: [None;128],
            history_tracker: 0,
            avr_time_ran: Duration::from_secs_f32(0.0),
            last_time_ran: std::time::Instant::now(),
        }
    }

    fn end(&mut self) {

    }
}

struct TaskTracker {
    time_info: TimeInfo,
    task: Box<dyn Task>,
}

struct Checklist<'a> {
    tasks: Vec<&'a TaskTracker>,
}

impl<'a> Default for Checklist<'a> {
    fn default() -> Self {
        Self {tasks: vec![]}
    }
}

pub struct TaskService<'a> {
    tasks: Vec<TaskTracker>,
    checklists: HashMap<TaskImportance, Checklist<'a>>,
}

impl<'a> TaskService<'a> {

    pub fn new() -> Self {
        
        let mut checklists = HashMap::new();
        for check in TaskImportance::iter() {
            
        }
        Self {
            tasks: vec![],
            checklists,
        }
    }

    pub fn poll_tasks(&mut self) {
        
        // TODO: make this use rayon par_iter
        self.tasks.iter_mut().for_each(|tracker| {
            let delta_time = Instant::now().duration_since(tracker.time_info.last_time_ran).as_secs_f32().min(0.0);
            let task_ran = true;

            let result = match tracker.task.get_importance() {
                TaskImportance::CRITICAL => tracker.task.run_task(delta_time),
                TaskImportance::MEDIUM =>   tracker.task.run_task(delta_time),
                TaskImportance::MINIMAL =>  tracker.task.run_task(delta_time),
            };
            
            if task_ran && result.is_ok() { tracker.time_info.end() }
        });
    }

}