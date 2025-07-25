// a system for scheduling tasks of varying importance,
// with a delta so less important tasks can be ignored to avoid lag spikes

use std::{sync::{self, mpsc::{Receiver, Sender}, Arc}, thread, time::{Duration, Instant}};

pub struct TaskMessenger {
    pub sender: Sender<FromTask>, 
    pub reciever: Receiver<ToTask>,
    pub self_sender: Sender<ToTask>,
}

pub trait Task {
    fn get_importance(&self) -> TaskType;    
    fn run_task(
        &mut self,
        message: &mut TaskMessenger,
        // the time since the function was ran last
        delta_time: f32,
    ) -> anyhow::Result<()>;
}

use strum_macros::EnumIter; 

#[derive(Debug, EnumIter)]
pub enum TaskType {
    LOOPING,
}

struct TimeInfo {
    running_time_history: [Option<std::time::Duration>; 128],
    history_tracker: usize,

    avr_time_ran: std::time::Duration,
    last_time_ran: std::time::Instant,
}

impl Default for TimeInfo {
    fn default() -> Self {
        Self {
            running_time_history: [None;128],
            history_tracker: 0,
            avr_time_ran: Duration::from_secs_f32(0.0),
            last_time_ran: std::time::Instant::now(),
        }
    }
}

impl TimeInfo {
    fn end(&mut self) {
        todo!();
    }
}

struct TaskTracker {
    time_info: TimeInfo,
    // transmit info
    sender: Sender<ToTask>,
    reciever: Receiver<FromTask>,
}

fn spawn_task(mut task: Box<dyn Task + Send>, self_sender: Sender<ToTask>, sender: Sender<FromTask>, reciever: Receiver<ToTask>) {
    
    thread::spawn(move || {
        let mut messages = TaskMessenger {sender, reciever, self_sender};
        let mut last_time_ran = Instant::now();
        while let Ok(message) = messages.reciever.recv() {
            match message {
                ToTask::Exit => return,
                ToTask::Schedule(at_time) => {
                    // sleep until it's the scheduled time.
                    thread::sleep(at_time.duration_since(Instant::now()));
                    // run the function
                    let _task_result = task.run_task(
                        &mut messages,
                        // way to much code, i know. i just don't care.
                        Instant::now().checked_duration_since(last_time_ran).unwrap_or(Duration::from_secs_f32(1.0/60.0)).as_secs_f32()
                    );
                    last_time_ran = Instant::now();
                }
            }
        }

    });
}

impl TaskTracker {
    fn new(task: Box<dyn Task + Send>) -> Self {
        
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

pub struct TaskService {
    // for sending and recieving data from adam
    send_adam: Sender<ToAdam>,
    recieve_adam: Receiver<FromAdam>,
}

pub enum FromTask {
    
}

pub enum ToTask {
    Exit,
    Schedule(std::time::Instant),
}

pub enum ToAdam {
    // stop adam
    Exit,
    
    AddTask(Box<dyn Task + Send>),
}
pub enum FromAdam {

}

struct Adam {
    adam_reciever: Receiver<ToAdam>, 
    adam_sender: Sender<FromAdam>,
    self_sender: Sender<ToAdam>,
    window: Arc<winit::window::Window>,

    tasks: Vec<TaskTracker>,
}

fn spawn_adam(window: Arc<winit::window::Window>, adam_reciever: Receiver<ToAdam>, adam_sender: Sender<FromAdam>, send_adam: Sender<ToAdam>) {
    thread::spawn(move || {
        
        let mut adam = Adam {
            window, 
            adam_reciever, 
            adam_sender, 
            self_sender: send_adam,
            tasks: vec![],
        };

        loop {
            match adam.adam_reciever.recv() {
                Ok(recieved) => {

                    match recieved {
                        ToAdam::Exit => {
                            println!("from adam: adam is closing.");
                            return;
                        },

                        ToAdam::AddTask(new_task) => {
                            let task_type = new_task.get_importance();
                            let task = TaskTracker::new(new_task);
                            
                            match task_type {
                                TaskType::LOOPING => task.sender.send(ToTask::Schedule(Instant::now())).unwrap(),
                            }

                            adam.tasks.push(task);
                        }
                    }

                },
                Err(error) => {println!("from adam: {}", error); return;},
            }
        }
    });
}

impl TaskService {

    pub fn add_tasks(&mut self, tasks: Vec<Box<dyn Task + Send>>) {
        for task in tasks {
            let _ = self.send_adam.send(ToAdam::AddTask(task));
        }
    }

    pub fn new(window: Arc<winit::window::Window>) -> Self {

        let (send_adam, adam_reciever) = sync::mpsc::channel::<ToAdam>();
        let (adam_sender, recieve_adam) = sync::mpsc::channel::<FromAdam>();

        spawn_adam(window.clone(), adam_reciever, adam_sender, send_adam.clone());

        Self {
            send_adam, recieve_adam,
        }
    }

    // this will close the eternal adam task, only do this when closing the entire program
    pub fn close_adam(&self) {

        let send_message = || self.send_adam.send(ToAdam::Exit);

        let mut tries = 0;
        let max_tries = 100;
        println!("attempting to close adam...");
        while let Err(adam_result) = send_message() {
            print!("\rfailed to close adam {} out of {} tries", tries, max_tries);
            tries+=1;
            if tries > max_tries {
                println!("after {} tries, the error: {}; was emmited", max_tries, adam_result);
                break;
            }
        }
        println!("sucessfully closed adam after {} tries.", tries);
    }

}