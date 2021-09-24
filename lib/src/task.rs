use super::error::Error;
use super::serde::{Deserialize, Serialize};
use super::typetag;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// In general the time delay is a very
/// simple implementation just based on the current
/// timestamp and a delay
/// For this simple usecase this is more than enough
pub type TimeMs = u128;

pub trait TimeSourceClone {
    fn box_clone(&self) -> Box<dyn TimeSource>;
}

impl<T> TimeSourceClone for T
where
    T: 'static + TimeSource + Clone,
{
    fn box_clone(&self) -> Box<dyn TimeSource> {
        Box::new(self.clone())
    }
}

/// Time source expects time to be returned in
/// ms
#[typetag::serde(tag = "type")]
pub trait TimeSource: TimeSourceClone + Send {
    fn get_time_ms(&mut self) -> TimeMs;
}

impl Clone for Box<dyn TimeSource> {
    fn clone(&self) -> Box<dyn TimeSource> {
        self.box_clone()
    }
}

/// dummy time source with pre-defined return values
/// for every time a time is requested
/// once again useful for demos or unit tests
#[derive(Clone, Serialize, Deserialize)]
pub struct InMemoryTimeSource {
    times: Vec<TimeMs>,
}

impl InMemoryTimeSource {
    pub fn new(times: Vec<TimeMs>) -> Self {
        Self { times }
    }
}

#[typetag::serde]
impl TimeSource for InMemoryTimeSource {
    fn get_time_ms(&mut self) -> TimeMs {
        self.times.pop().unwrap_or(0)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TimeSourceTypes {
    Clock(ClockTimeSource),
    InMemory(InMemoryTimeSource),
    Generic(Box<dyn TimeSource>),
}

#[typetag::serde]
impl TimeSource for TimeSourceTypes {
    fn get_time_ms(&mut self) -> TimeMs {
        match self {
            Self::Clock(ts) => ts.get_time_ms(),
            Self::InMemory(ts) => ts.get_time_ms(),
            Self::Generic(ts) => ts.get_time_ms(),
        }
    }
}

/// actual time-based source
/// this is usually the source you want
#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone, Serialize, Deserialize)]
pub struct Task {
    repeat: bool,
    done: bool,
    delay: TimeMs,
    start: TimeMs,
    time_src: TimeSourceTypes,
}

impl Task {
    pub fn new(repeat: bool, delay: TimeMs, mut time_src: TimeSourceTypes) -> Self {
        Self {
            repeat,
            done: false,
            delay,
            start: time_src.get_time_ms(),
            time_src,
        }
    }

    pub fn from_str(
        repeat: bool,
        time_str: &str,
        time_src: TimeSourceTypes,
    ) -> Result<Self, Error> {
        Ok(Self::new(repeat, Self::scan(time_str)?, time_src))
    }

    /// scans a string of the following format
    /// 1h30m45s inot a timestamp
    pub fn scan(time_str: &str) -> Result<TimeMs, Error> {
        let mut start;
        let mut current = 0;
        let mut result = 0;

        let mut operators = HashMap::new();
        operators.insert("\0", 1000);
        operators.insert("ms", 1);
        operators.insert("s", 1000);
        operators.insert("m", 60000);
        operators.insert("h", 3600000);

        // always scan number+operator
        while current < time_str.len() {
            start = current;
            // TODO ignore white-spaces between numbers/operators
            let num = Self::scan_num(time_str, start, &mut current)?;

            start = current;
            let op = Self::scan_operator(time_str, start, &mut current, &operators)?;
            result += num * op;
        }
        Ok(result)
    }

    fn scan_operator(
        time_str: &str,
        start: usize,
        current: &mut usize,
        operators: &HashMap<&str, TimeMs>,
    ) -> Result<TimeMs, Error> {
        while Self::is_alpha(time_str.chars().nth(*current).unwrap_or('\0')) {
            *current += 1;
        }
        // no operator? return 1, but only if
        // current is end of string too
        if start == *current && *current >= time_str.len() {
            Ok(1)
        } else if start == *current {
            Err(Error::TimeStringUnknownOperator)
        } else {
            let operator = &time_str[start..*current];
            match operators.get(operator) {
                Some(value) => Ok(*value),
                _ => Err(Error::TimeStringUnknownOperator),
            }
        }
    }

    fn scan_num(time_str: &str, start: usize, current: &mut usize) -> Result<TimeMs, Error> {
        while Self::is_numeric(time_str.chars().nth(*current).unwrap_or('\0')) {
            *current += 1;
        }

        let number = &time_str[start..*current];
        Ok(TimeMs::from_str_radix(number, 10)?)
    }

    pub fn is_numeric(c: char) -> bool {
        ('0'..='9').contains(&c)
    }

    pub fn is_alpha(c: char) -> bool {
        ('a'..='z').contains(&c) || ('A'..='Z').contains(&c)
    }

    pub fn next_time(&self) -> u128 {
        self.start + self.delay
    }

    pub fn is_due(&mut self) -> bool {
        if !self.done && self.next_time() < self.time_src.get_time_ms() {
            self.done = !self.repeat; // if no repeate set to done
            self.start = self.time_src.get_time_ms(); // next start time
            true
        } else {
            false
        }
    }

    pub fn get_time_str(&self) -> String {
        let mut remainder = self.delay;
        let mut result = "".to_string();

        let operators = vec![("h", 3600000), ("m", 60000), ("s", 1000), ("ms", 1)];

        for (key, value) in operators {
            result = format!("{}{}{}", result, remainder / value, key);
            remainder %= value;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_not_trigger() {
        let mut t = Task::new(
            true,
            10,
            TimeSourceTypes::InMemory(InMemoryTimeSource::new(vec![101, 101, 100])),
        );

        assert!(!t.done);
        assert_eq!(t.start, 100);
        assert!(!t.is_due());
        assert!(!t.done);
        assert_eq!(t.start, 100);
    }

    #[test]
    fn it_should_trigger_and_repeat() {
        let mut t = Task::new(
            true,
            10,
            TimeSourceTypes::InMemory(InMemoryTimeSource::new(vec![111, 111, 100])),
        );

        assert!(!t.done);
        assert_eq!(t.start, 100);
        assert!(t.is_due());
        assert!(!t.done);
        assert_eq!(t.start, 111);
    }

    #[test]
    fn it_should_trigger_and_not_repeat() {
        let mut t = Task::new(
            false,
            10,
            TimeSourceTypes::InMemory(InMemoryTimeSource::new(vec![122, 111, 111, 100])),
        );

        assert!(!t.done);
        assert_eq!(t.start, 100);
        assert!(t.is_due());
        assert!(t.done);
        assert_eq!(t.start, 111);
        assert!(!t.is_due());
    }

    #[test]
    fn it_should_parse_time_str() {
        let ms = Task::scan("1h20m10s5").unwrap();
        assert_eq!(ms, (1 * 3600000) + (20 * 60000) + (10 * 1000) + 5);
    }

    #[test]
    fn it_should_not_parse_time_str_bad_operator() {
        assert_eq!(Task::scan("1h20m10@5").unwrap_or(0), 0);
    }

    #[test]
    fn it_should_revert_time_str() {
        let ms = Task::from_str(
            false,
            "1h20m10s5",
            TimeSourceTypes::InMemory(InMemoryTimeSource::new(vec![])),
        )
        .unwrap();
        assert_eq!(ms.get_time_str(), "1h20m10s5ms".to_string());
    }
}
