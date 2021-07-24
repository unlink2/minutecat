use super::logset::LogSet;
use super::logfile::Logfile;
use super::source::{DataSource, HttpDataSource, FileDataSource};
use super::task::{Task, ClockTimeSource};
use super::error::BoxResult;
use super::trigger::{Trigger, RegexTrigger, TriggerType};

/// a command is an action that modifies a struct
/// it has the property of offering a detached setter mechanism
/// that can be e.g. used in queues
/// it also offers an undo feature
/// this should be used instead of directly mutating
/// a struct because it offers a more structured way
/// of executing tasks
///
/// A command should always be self-contained, act upon a single struct
/// and not contain direct references (e.g. only data).
/// That way commands could be applied to one or more similar objects.
/// They are pure data-containers!
///
/// Because they are self-contained they can easily be tested as well!
pub trait Command<T> {
    fn execute(&mut self, obj: &mut T) -> BoxResult<()>;
    fn undo(&mut self, obj: &mut T) -> BoxResult<()>;
}

pub enum FileType {
    Http,
    Local
}

pub struct AddFileCommand {
    pub name: String,
    pub location: String,
    pub line_limit: usize,
    pub refresh_time: String,
    pub file_type: FileType,
    pub can_undo: bool
}

impl AddFileCommand {
    pub fn new(name: &str,
        location: &str,
        line_limit: usize,
        refresh_time: &str,
        file_type: FileType) -> Self {
        Self {
            name: name.into(),
            location: location.into(),
            line_limit,
            refresh_time: refresh_time.into(),
            can_undo: false,
            file_type
        }
    }
}

impl Command<LogSet> for AddFileCommand {
    fn execute(&mut self, logset: &mut LogSet) -> BoxResult<()> {
        let ds: Box<dyn DataSource> = match self.file_type {
            FileType::Local => Box::new(FileDataSource::new(&self.location, self.line_limit)),
            FileType::Http => Box::new(HttpDataSource::new(&self.location))
        };
        logset.logs.push(Logfile::new(&self.name, ds,
                Task::from_str(true, &self.refresh_time, Box::new(ClockTimeSource))?
            ));
        self.can_undo = true;
        Ok(())
    }

    fn undo(&mut self, logset: &mut LogSet) -> BoxResult<()> {
        if self.can_undo {
            self.can_undo = false;
            logset.pop();
        }
        Ok(())
    }
}

pub struct DeleteLogfileCommand {
    pub index: usize,
    pub removed: Option<Logfile>
}

impl DeleteLogfileCommand {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            removed: None
        }
    }
}

impl Command<LogSet> for DeleteLogfileCommand {
    fn execute(&mut self, logset: &mut LogSet) -> BoxResult<()> {
        self.removed = logset.remove(self.index);
        Ok(())
    }

    fn undo(&mut self, logset: &mut LogSet) -> BoxResult<()> {
        match &self.removed {
            Some(logfile) => {
                logset.push(logfile.clone());
                self.removed = None;
            }
            _ => {}
        }
        Ok(())
    }
}

pub struct AddRegexTriggerCommand {
    name: String,
    desc: String,
    trigger_type: TriggerType,
    regex: String,
    invert: bool,
    can_undo: bool
}

impl AddRegexTriggerCommand {
    pub fn new(
        name: &str,
        desc: &str,
        trigger_type: TriggerType,
        regex: &str,
        invert: bool) -> Self {
        Self {
            name: name.into(),
            desc: desc.into(),
            trigger_type,
            regex: regex.into(),
            invert,
            can_undo: false
        }
    }
}

impl Command<Logfile> for AddRegexTriggerCommand {
    fn execute(&mut self, log: &mut Logfile) -> BoxResult<()> {
        log.push(Box::new(RegexTrigger::new(
                    &self.name,
                    &self.desc,
                    self.trigger_type,
                    &self.regex,
                    self.invert)));
        self.can_undo = true;
        Ok(())
    }

    fn undo(&mut self, log: &mut Logfile) -> BoxResult<()> {
        if self.can_undo {
            log.pop();
            self.can_undo = false;
        }
        Ok(())
    }
}

pub struct RemoveTriggerCommand {
    index: usize,
    removed: Option<Box<dyn Trigger>>
}

impl RemoveTriggerCommand {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            removed: None
        }
    }
}

impl Command<Logfile> for RemoveTriggerCommand {
    fn execute(&mut self, log: &mut Logfile) -> BoxResult<()> {
        self.removed = log.remove(self.index);
        Ok(())
    }

    fn undo(&mut self, log: &mut Logfile) -> BoxResult<()> {
        match &self.removed {
            Some(trigger) => {
                log.push(trigger.clone());
                self.removed = None;
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::InMemoryDataSource;

    #[test]
    fn it_should_execute_add_cmd() {
        let mut ls = LogSet::new();

        let mut cmd1 = AddFileCommand::new("name", "local/test", 100, "1h", FileType::Local);
        let mut cmd2 = AddFileCommand::new("name", "local/test", 100, "1h", FileType::Local);

        // ad twice undo one
        cmd1.execute(&mut ls).unwrap();
        cmd2.execute(&mut ls).unwrap();
        assert_eq!(ls.len(), 2);

        // should always undo in same order!
        cmd2.undo(&mut ls).unwrap();
        assert_eq!(ls.len(), 1);

        // same command should not undo again!
        cmd2.undo(&mut ls).unwrap();
        assert_eq!(ls.len(), 1);

        // cmd1 should
        cmd1.undo(&mut ls).unwrap();
        assert_eq!(ls.len(), 0);
    }

    #[test]
    fn it_should_delete_cmd() {
        let mut ls = LogSet::new();

        let mut cmd1 = AddFileCommand::new("name", "local/test", 100, "1h", FileType::Local);
        let mut cmd2 = AddFileCommand::new("name", "local/test", 100, "1h", FileType::Local);

        // ad twice undo one
        cmd1.execute(&mut ls).unwrap();
        cmd2.execute(&mut ls).unwrap();
        assert_eq!(ls.len(), 2);

        let mut delcmd = DeleteLogfileCommand::new(1);
        delcmd.execute(&mut ls).unwrap();
        assert_eq!(ls.len(), 1);

        // undo it!
        delcmd.undo(&mut ls).unwrap();
        assert_eq!(ls.len(), 2);
    }

    #[test]
    fn it_should_add_re_trigger() {
        let mut l = Logfile::new("name",
            Box::new(InMemoryDataSource::new(vec![])),
            Task::new(false, 0, Box::new(ClockTimeSource)));

        let mut cmd = AddRegexTriggerCommand::new("name", "desc", TriggerType::NoEvent, "", false);
        cmd.execute(&mut l).unwrap();

        assert_eq!(l.len(), 1);

        cmd.undo(&mut l).unwrap();
        assert_eq!(l.len(), 0);
    }

    #[test]
    fn it_should_remove_re_trigger() {
        let mut l = Logfile::new("name",
            Box::new(InMemoryDataSource::new(vec![])),
            Task::new(false, 0, Box::new(ClockTimeSource)));

        let mut cmd = AddRegexTriggerCommand::new("name", "desc", TriggerType::NoEvent, "", false);
        let mut delcmd = RemoveTriggerCommand::new(0);
        cmd.execute(&mut l).unwrap();
        assert_eq!(l.len(), 1);

        delcmd.execute(&mut l).unwrap();
        assert_eq!(l.len(), 0);

        delcmd.undo(&mut l).unwrap();
        assert_eq!(l.len(), 1);

        delcmd.undo(&mut l).unwrap();
        assert_eq!(l.len(), 1);
    }
}
