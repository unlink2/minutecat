use super::source::DataSource;
use super::trigger::Trigger;
use super::task::Task;
use super::serde::{Serialize, Deserialize};
use super::error::BoxResult;

/// An event handler callback
/// that is notified whenever a text trigger is true
pub trait EventHandler {
    fn on_event(&mut self, trigger: &dyn Trigger, text: &str);
}

#[derive(Serialize, Deserialize)]
pub struct Logfile {
    pub name: String,
    pub text: String,
    source: Box<dyn DataSource>,
    pub triggers: Vec<Box<dyn Trigger>>,
    pub task: Task,
}

impl Logfile {
    pub fn new(name: &str, source: Box<dyn DataSource>, task: Task) -> Self {
        Self {
            name: name.into(),
            text: "".into(),
            source,
            triggers: vec![],
            task
        }
    }

    pub fn push(&mut self, trigger: Box<dyn Trigger>) {
        self.triggers.push(trigger);
    }

    pub fn pop(&mut self) -> Option<Box<dyn Trigger>> {
        self.triggers.pop()
    }

    pub fn remove(&mut self, index: usize) -> Option<Box<dyn Trigger>> {
        if self.triggers.len() > index {
            Some(self.triggers.remove(index))
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.triggers.len()
    }

    /// call this to update
    /// a logfile based on the task timer
    /// and the data source origin
    /// returns trigger results in a vec
    pub fn update(&mut self, handlers: &mut Vec<&mut dyn EventHandler>) -> BoxResult<bool> {
        // is it ready to update?
        if !self.task.is_due() {
            return Ok(false);
        }

        // if so refresh source
        self.text = self.source.as_mut().load()?;

        self.check(handlers)?;

        return Ok(true);
    }

    pub fn check(&self, handlers: &mut Vec<&mut dyn EventHandler>) -> BoxResult<()> {
        // and check triggers
        for trigger in &self.triggers[..] {
            if trigger.check(&self.text)? {
                self.notify(trigger.as_ref(), handlers);
            }
        }
        return Ok(());
    }

    fn notify(&self, trigger: &dyn Trigger, handlers: &mut Vec<&mut dyn EventHandler>) {
        for handler in &mut handlers[..] {
            handler.on_event(trigger, &self.text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::InMemoryDataSource;
    use crate::task::InMemoryTimeSource;
    use crate::trigger::{RegexTrigger, TriggerType};

    struct TestHandler(Option<TriggerType>);
    impl EventHandler for TestHandler {
        fn on_event(&mut self, trigger: &dyn Trigger, _text: &str) {
            self.0 = Some(trigger.get_type());
        }
    }

    #[test]
    fn it_should_trigger_and_call_handlers() {
        let mut lf = Logfile::new(
            "test",
            Box::new(InMemoryDataSource::new(vec![
                "Test data with error".into(),
                "Original test data success".into()])),
            Task::new(true, 10,
                Box::new(InMemoryTimeSource::new(vec![133, 122, 122, 111])))
        );

        lf.push(Box::new(RegexTrigger::new("success", "on success",
                    TriggerType::Success, "success")));
        lf.push(Box::new(RegexTrigger::new("failure", "on error",
                    TriggerType::Error, "error")));

        let mut handler = TestHandler(None);
        // test handlers
        lf.update(&mut vec![&mut handler]).unwrap();
        assert_eq!(handler.0, Some(TriggerType::Success));

        lf.update(&mut vec![&mut handler]).unwrap();
        assert_eq!(handler.0, Some(TriggerType::Error));
    }

    #[test]
    fn it_should_not_trigger() {
        let mut lf = Logfile::new(
            "test",
            Box::new(InMemoryDataSource::new(vec![
                "Test data with error".into(),
                "Original test data success".into()])),
            Task::new(true, 10,
                Box::new(InMemoryTimeSource::new(vec![111, 111, 111, 111])))
        );

        lf.push(Box::new(RegexTrigger::new("success", "on success",
                    TriggerType::Success, "success")));
        lf.push(Box::new(RegexTrigger::new("failure", "on error",
                    TriggerType::Error, "error")));

        let mut handler = TestHandler(None);
        // test handlers
        lf.update(&mut vec![&mut handler]).unwrap();
        assert_eq!(handler.0, None);
    }
}
