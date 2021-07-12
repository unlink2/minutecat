use super::source::DataSource;
use super::trigger::Trigger;
use super::task::Task;
use super::serde::{Serialize, Deserialize};
use super::error::BoxResult;

/// An event handler callback
/// that is notified whenever a text trigger is true
type EventHandler = fn(&dyn Trigger, &str) -> ();

#[derive(Serialize, Deserialize)]
pub struct Logfile {
    text: String,
    source: Box<dyn DataSource>,
    triggers: Vec<Box<dyn Trigger>>,
    task: Task,
}

impl Logfile {
    pub fn new(source: Box<dyn DataSource>, task: Task) -> Self {
        Self {
            text: "".into(),
            source,
            triggers: vec![],
            task
        }
    }

    /// call this to update
    /// a logfile based on the task timer
    /// and the data source origin
    /// returns trigger results in a vec
    pub fn update(&mut self, handlers: &Vec<EventHandler>) -> BoxResult<bool> {
        // is it ready to update?
        if !self.task.is_due() {
            return Ok(false);
        }

        // if so refresh source
        self.text = self.source.as_mut().load()?;

        self.check(handlers);

        return Ok(true);
    }

    pub fn check(&self, handlers: &Vec<EventHandler>) {
        // and check triggers
        for trigger in &self.triggers[..] {
            if trigger.check(&self.text) {
                self.notify(trigger.as_ref(), handlers);
            }
        }
    }

    fn notify(&self, trigger: &dyn Trigger, handlers: &Vec<EventHandler>) {
        for handler in &handlers[..] {
                handler(trigger, &self.text);
        }
    }
}
