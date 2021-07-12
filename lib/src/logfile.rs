use super::source::DataSource;
use super::trigger::Trigger;
use super::task::Task;
use super::serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub struct Logfile {
    text: String,
    source: Box<dyn DataSource>,
    triggers: Vec<Box<dyn Trigger>>,
    task: Task
}

impl Logfile {
    pub fn new(source: Box<dyn DataSource>, triggers: Vec<Box<dyn Trigger>>, task: Task) -> Self {
        Self {
            text: "".into(),
            source,
            triggers,
            task
        }
    }
}
