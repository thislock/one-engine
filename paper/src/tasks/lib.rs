use crate::engine::Engine;


mod queue_render;

pub fn init_tasks(engine: &mut Engine) {
    engine.task_service.add_tasks(
        vec![
            Box::new(queue_render::QueueRender {window: engine.get_window()}),
        ]
    );
}