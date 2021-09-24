use super::error::Error;
use super::extra::ExtraData;
use super::serde::{Deserialize, Serialize};
use super::source::{DataSource, DataSourceTypes};
use super::task::Task;
use super::trigger::{Trigger, TriggerTypes};
use std::fmt;

/// An event handler callback
/// that is notified whenever a text trigger is true
pub trait EventHandler {
    /// called when the trigger fires
    fn on_event(&mut self, event: &Event);
}

pub struct Event<'a> {
    pub did_trigger: bool,
    pub trigger: Option<&'a dyn Trigger>,
    pub task: &'a Task,
    pub extra: &'a mut ExtraData,
    pub text: &'a str,
    pub name: &'a str,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Logfile {
    pub name: String,
    source: DataSourceTypes,
    pub triggers: Vec<TriggerTypes>,
    pub task: Task,
    /// extra data may be used by EventHandlers to store data
    #[serde(default)]
    pub extra: ExtraData,
}

impl PartialEq for Logfile {
    // TODO maybe implement eq for triggers, source and task
    fn eq(&self, other: &Logfile) -> bool {
        self.name == other.name && self.triggers.len() == other.triggers.len()
    }
}

impl fmt::Debug for Logfile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Logfile {
    pub fn new(name: &str, source: DataSourceTypes, task: Task) -> Self {
        Self {
            name: name.into(),
            source,
            triggers: vec![],
            task,
            extra: ExtraData::new(),
        }
    }

    pub fn push(&mut self, trigger: TriggerTypes) {
        self.triggers.push(trigger);
    }

    pub fn pop(&mut self) -> Option<TriggerTypes> {
        self.triggers.pop()
    }

    pub fn remove(&mut self, index: usize) -> Option<TriggerTypes> {
        if self.triggers.len() > index {
            Some(self.triggers.remove(index))
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.triggers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// call this to update
    /// a logfile based on the task timer
    /// and the data source origin
    /// returns trigger results in a vec
    /// Note that currently update may call long blocking IO operations
    /// and is therefore best used in a thread.
    /// There might be an async version in the future.
    pub async fn update(
        &mut self,
        handlers: &mut Vec<&mut dyn EventHandler>,
    ) -> Result<bool, Error> {
        // is it ready to update?
        if !self.task.is_due() {
            return Ok(false);
        }
        self.force_update(handlers).await
    }

    pub async fn force_update(
        &mut self,
        handlers: &mut Vec<&mut dyn EventHandler>,
    ) -> Result<bool, Error> {
        // if so refresh source
        let text = self.source.load().await?;

        self.check(handlers, &text)?;

        Ok(true)
    }

    pub fn check(
        &mut self,
        handlers: &mut Vec<&mut dyn EventHandler>,
        text: &str,
    ) -> Result<(), Error> {
        // and check triggers

        if self.triggers.is_empty() {
            let event = Event {
                did_trigger: false,
                trigger: None,
                task: &self.task,
                extra: &mut self.extra,
                text,
                name: &self.name,
            };
            for handler in &mut handlers[..] {
                handler.on_event(&event);
            }
        } else {
            for trigger in &self.triggers[..] {
                let event = Event {
                    did_trigger: trigger.check(text)?,
                    trigger: Some(trigger),
                    task: &self.task,
                    extra: &mut self.extra,
                    text,
                    name: &self.name,
                };
                for handler in &mut handlers[..] {
                    handler.on_event(&event);
                }
            }
        }
        Ok(())
    }
}

// TODO test push,pop and remove
// TODO test update
#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::InMemoryDataSource;
    use crate::task::InMemoryTimeSource;
    use crate::task::TimeSourceTypes;
    use crate::trigger::{RegexTrigger, TriggerType};

    struct TestHandler(Option<TriggerType>, Option<TriggerType>);
    impl EventHandler for TestHandler {
        fn on_event(&mut self, event: &Event) {
            if event.did_trigger {
                self.0 = Some(event.trigger.unwrap().get_type());
            } else {
                self.1 = Some(event.trigger.unwrap().get_type());
            }
        }
    }

    #[tokio::test]
    async fn it_should_trigger_and_call_handlers() {
        let mut lf = Logfile::new(
            "test",
            DataSourceTypes::InMemory(InMemoryDataSource::new(vec![
                "Test data with error".into(),
                "Original test data success".into(),
            ])),
            Task::new(
                true,
                10,
                TimeSourceTypes::InMemory(InMemoryTimeSource::new(vec![133, 122, 122, 111])),
            ),
        );

        lf.push(TriggerTypes::Regex(RegexTrigger::new(
            "success",
            "on success",
            TriggerType::Success,
            "success",
            false,
        )));
        lf.push(TriggerTypes::Regex(RegexTrigger::new(
            "failure",
            "on error",
            TriggerType::Error,
            "error",
            false,
        )));

        let mut handler = TestHandler(None, None);
        // test handlers
        lf.update(&mut vec![&mut handler]).await.unwrap();
        assert_eq!(handler.0, Some(TriggerType::Success));
        assert_eq!(handler.1, Some(TriggerType::Error));

        lf.update(&mut vec![&mut handler]).await.unwrap();
        assert_eq!(handler.0, Some(TriggerType::Error));
        assert_eq!(handler.1, Some(TriggerType::Success));
    }

    #[tokio::test]
    async fn it_should_not_trigger() {
        let mut lf = Logfile::new(
            "test",
            DataSourceTypes::InMemory(InMemoryDataSource::new(vec![
                "Test data with error".into(),
                "Original test data success".into(),
            ])),
            Task::new(
                true,
                10,
                TimeSourceTypes::InMemory(InMemoryTimeSource::new(vec![111, 111, 111, 111])),
            ),
        );

        lf.push(TriggerTypes::Regex(RegexTrigger::new(
            "success",
            "on success",
            TriggerType::Success,
            "success",
            false,
        )));
        lf.push(TriggerTypes::Regex(RegexTrigger::new(
            "failure",
            "on error",
            TriggerType::Error,
            "error",
            false,
        )));

        let mut handler = TestHandler(None, None);
        // test handlers
        lf.update(&mut vec![&mut handler]).await.unwrap();
        assert_eq!(handler.0, None);
        assert_eq!(handler.1, None);
    }
}
