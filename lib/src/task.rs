use super::serde::{Serialize, Deserialize};
use super::typetag;
use std::time::{SystemTime, UNIX_EPOCH};

/// In general the time delay is a very
/// simple implementation just based on the current
/// timestamp and a delay
/// For this simple usecase this is more than enough
pub type TimeMs = u128;

/// Time source expects time to be returned in
/// ms
#[typetag::serde(tag = "type")]
pub trait TimeSource {
    fn get_time_ms(&mut self) -> TimeMs;
}

/// dummy time source with pre-defined return values
/// for every time a time is requested
/// once again useful for demos or unit tests
#[derive(Serialize, Deserialize)]
pub struct InMemoryTimeSource {
    times: Vec<TimeMs>
}

impl InMemoryTimeSource {
    pub fn new(times: Vec<TimeMs>) -> Self {
        Self {times}
    }
}

#[typetag::serde]
impl TimeSource for InMemoryTimeSource {
    fn get_time_ms(&mut self) -> TimeMs {
        self.times.pop().unwrap_or(0)
    }
}

/// actual time-based source
/// this is usually the source you want
#[derive(Serialize, Deserialize)]
pub struct ClockTimeSource;

#[typetag::serde]
impl TimeSource for ClockTimeSource {
    fn get_time_ms(&mut self) -> TimeMs {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }
}



#[derive(Serialize, Deserialize)]
pub struct Task {
    repeat: bool,
    done: bool,
    delay: TimeMs,
    start: TimeMs,
    time_src: Box<dyn TimeSource>
}

impl Task {
    pub fn new(repeat: bool, delay: TimeMs, mut time_src: Box<dyn TimeSource>) -> Self {
        Self {
            repeat,
            done: false,
            delay,
            start: time_src.get_time_ms(),
            time_src
        }
    }

    pub fn is_due(&mut self) -> bool {
        if !self.done
            && self.start+self.delay < self.time_src.get_time_ms() {
            self.done = !self.repeat; // if no repeate set to done
            self.start = self.time_src.get_time_ms(); // next start time
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_not_trigger() {
        let mut t = Task::new(true, 10, Box::new(InMemoryTimeSource::new(vec![101, 101, 100])));

        assert!(!t.done);
        assert_eq!(t.start, 100);
        assert!(!t.is_due());
        assert!(!t.done);
        assert_eq!(t.start, 100);
    }

    #[test]
    fn it_should_trigger_and_repeat() {
        let mut t = Task::new(true, 10, Box::new(InMemoryTimeSource::new(vec![111, 111, 100])));

        assert!(!t.done);
        assert_eq!(t.start, 100);
        assert!(t.is_due());
        assert!(!t.done);
        assert_eq!(t.start, 111);
    }

    #[test]
    fn it_should_trigger_and_not_repeat() {
        let mut t = Task::new(false, 10, Box::new(InMemoryTimeSource::new(vec![122, 111, 111, 100])));

        assert!(!t.done);
        assert_eq!(t.start, 100);
        assert!(t.is_due());
        assert!(t.done);
        assert_eq!(t.start, 111);
        assert!(!t.is_due());
    }
}
