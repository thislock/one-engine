// a system for scheduling tasks of varying importance,
// with a delta so less important tasks can be ignored to avoid lag spikes

use std::{
  sync::{
    self,
    mpsc::{Receiver, Sender},
  },
  thread,
  time::{Duration, Instant},
};

#[allow(unused)]
pub struct TaskMessenger {
  pub sender: Sender<FromTask>,
  pub reciever: Receiver<ToTask>,
  pub self_sender: Sender<ToTask>,
}

pub trait Task {
  fn get_type(&self) -> TaskType;
  fn run_task(
    &mut self,
    message: &mut TaskMessenger,
    // the time since the function was ran last
    delta_time: f32,
  ) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct LoopGroup {
  send: Sender<Sender<ToTask>>,
}

fn spawn_loop_group(loop_epoch: Duration, rx: Receiver<Sender<ToTask>>) {
  thread::spawn(move || {
    const FUCK: &str = "failed to send message in loop group main loop";

    let channel = rx;
    let mut loop_channels = vec![];

    let mut alive = true;

    let loop_epoch = loop_epoch;

    while alive {
      // check if there are any new victims
      match channel.try_recv() {
        Ok(new_victim) => loop_channels.push(new_victim),
        _ => {}
      }
      // go through every victim and run them
      let now = Instant::now();
      let mut errors = vec![];
      for victim in &loop_channels {
        let result = victim.send(ToTask::Schedule(now));
        if let Err(err) = result {
          errors.push(err);
        }
      }

      // check for erros
      if errors.len() != 0 {
        for error in errors {
          paper_error::log_error(FUCK, error.into());
        }
        alive = false;
        paper_error::log("loop group closed after previous errors");
      }

      // wait until the next frame
      thread::sleep(loop_epoch);
    }
  });
}

impl LoopGroup {
  pub fn new(loop_epoch: Duration) -> Self {
    let (tx, rx) = sync::mpsc::channel::<Sender<ToTask>>();

    spawn_loop_group(loop_epoch, rx);

    Self { send: tx }
  }

  pub fn add_member(&self, sent: Sender<ToTask>) -> anyhow::Result<()> {
    self.send.send(sent)?;
    Ok(())
  }
}

use crate::{paper_error, translate_surface::SyncWindow};

#[derive(Debug)]
#[allow(unused)]
pub enum TaskType {
  Rare,
  Looping(LoopGroup),
}

#[allow(unused)]
struct TimeInfo {
  running_time_history: [Option<std::time::Duration>; 128],
  history_tracker: usize,

  avr_time_ran: std::time::Duration,
  last_time_ran: std::time::Instant,
}

impl Default for TimeInfo {
  fn default() -> Self {
    Self {
      running_time_history: [None; 128],
      history_tracker: 0,
      avr_time_ran: Duration::from_secs_f32(0.0),
      last_time_ran: std::time::Instant::now(),
    }
  }
}

#[allow(unused)]
impl TimeInfo {
  fn end(&mut self) {
    todo!();
  }
}

// ******************************************************************** //
// *************************** TASK TRACKER *************************** //
// ******************************************************************** //

#[allow(unused)]
struct TaskTracker {
  time_info: TimeInfo,
  // transmit info
  sender: Sender<ToTask>,
  reciever: Receiver<FromTask>,
}

impl TaskTracker {
  fn new(task: Box<dyn Task + Send>) -> Self {
    // create channels so both the task, and adam can communicate
    let (tx1, rx1) = sync::mpsc::channel::<ToTask>();
    let (tx2, rx2) = sync::mpsc::channel::<FromTask>();

    spawn_task(task, tx1.clone(), tx2, rx1);

    Self {
      time_info: TimeInfo::default(),
      sender: tx1,
      reciever: rx2,
    }
  }
}

fn spawn_task(
  mut task: Box<dyn Task + Send>,
  self_sender: Sender<ToTask>,
  sender: Sender<FromTask>,
  reciever: Receiver<ToTask>,
) {
  thread::spawn(move || {
    let mut messages = TaskMessenger {
      sender,
      reciever,
      self_sender,
    };
    let mut last_time_ran = Instant::now();

    while let Ok(message) = messages.reciever.recv() {
      match message {
        ToTask::Exit => break,
        ToTask::Schedule(at_time) => {
          // sleep until it's the scheduled time.
          thread::sleep(at_time.duration_since(Instant::now()));
          // run the function
          let _task_result = task.run_task(
            &mut messages,
            // way to much code, i know. i just don't care.
            Instant::now()
              .checked_duration_since(last_time_ran)
              .unwrap()
              .as_secs_f32(),
          );
          last_time_ran = Instant::now();
        }
      }
    }
  });
}

// ******************************************************************** //
// *************************** TASK SERVICE *************************** //
// ******************************************************************** //

// spawns a eternal task that manages every other task
#[allow(unused)]
pub struct TaskService {
  // for sending and recieving data from adam
  send_adam: Sender<ToAdam>,
  recieve_adam: Receiver<FromAdam>,
}

impl TaskService {
  pub fn add_tasks(&mut self, tasks: Vec<Box<dyn Task + Send>>) {
    for task in tasks {
      let _ = self.send_adam.send(ToAdam::AddTask(task));
    }
  }

  pub fn new(window: SyncWindow) -> Self {
    let (send_adam, adam_reciever) = sync::mpsc::channel::<ToAdam>();
    let (adam_sender, recieve_adam) = sync::mpsc::channel::<FromAdam>();

    spawn_task_master(window, adam_reciever, adam_sender, send_adam.clone());

    Self {
      send_adam,
      recieve_adam,
    }
  }
}

// ******************************************************************** //
// *************************** TASK MASTER **************************** //
// ******************************************************************** //

#[allow(unused)]
struct TaskMaster {
  task_reciever: Receiver<ToAdam>,
  task_sender: Sender<FromAdam>,
  self_sender: Sender<ToAdam>,
  window: SyncWindow,

  tasks: Vec<TaskTracker>,
}

fn spawn_task_master(
  window: SyncWindow,
  task_reciever: Receiver<ToAdam>,
  task_sender: Sender<FromAdam>,
  self_sender: Sender<ToAdam>,
) {
  thread::spawn(move || {
    let mut task_master = TaskMaster {
      window,
      task_reciever,
      task_sender,
      self_sender,
      tasks: vec![],
    };
    while let Ok(recieved) = task_master.task_reciever.recv() {
      match recieved {
        ToAdam::Exit => break,

        ToAdam::AddTask(new_task) => {
          let task_type = new_task.get_type();
          let task = TaskTracker::new(new_task);

          match task_type {
            TaskType::Looping(loop_group) => {
              let msg = loop_group.add_member(task.sender.clone());
              if let Err(err) = msg {
                paper_error::log_error("fucking fuck fuck", err);
              }
            }
            // typically just called by other tasks
            TaskType::Rare => {}
          }
          task_master.tasks.push(task);
        }
      }
    }
    // send every task the exit signal
    for task in task_master.tasks {
      let _ = task.sender.send(ToTask::Exit);
    }
  });
}

// ****************************************************************************** //
// ********************************* TASK ENUMS ********************************* //
// ****************************************************************************** //
pub enum FromTask {}

#[allow(unused)]
pub enum ToTask {
  Exit,
  Schedule(std::time::Instant),
}

#[allow(unused)]
pub enum ToAdam {
  // stop adam
  Exit,

  AddTask(Box<dyn Task + Send>),
}
pub enum FromAdam {}
