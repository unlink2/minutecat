use super::serde::{Serialize, Deserialize};
use super::typetag;
use super::error::BoxResult;
use super::regex::Regex;

pub trait TriggerClone {
    fn box_clone(&self) -> Box<dyn Trigger>;
}

impl<T> TriggerClone for T
where T: 'static + Trigger + Clone {
    fn box_clone(&self) -> Box<dyn Trigger> {
        Box::new(self.clone())
    }
}

/// TriggerType describes if a trigger
/// should be interpreted as an error,
/// a warning or success
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum TriggerType {
    NoEvent,
    Success,
    Warning,
    Error
}

/// A trigger is anything that can
/// cause a logfile notification to appear
/// e.g. regex match, time since last change
#[typetag::serde(tag = "type")]
pub trait Trigger: TriggerClone + Send {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn check(&self, text: &str) -> BoxResult<bool>;

    /// returns the slice that fired the trigger
    fn slice<'a>(&self, text: &'a str) -> BoxResult<&'a str>;
    fn get_type(&self) -> TriggerType;
}

impl Clone for Box<dyn Trigger> {
    fn clone(&self) -> Box<dyn Trigger> {
        self.box_clone()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RegexTrigger {
    name: String,
    description: String,
    trigger_type: TriggerType,
    re: String,

    #[serde(default)]
    invert: bool
}

impl RegexTrigger {
    pub fn new(name: &str, description: &str, trigger_type: TriggerType, re: &str, invert: bool) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            trigger_type,
            re: re.into(),
            invert
        }
    }
}

#[typetag::serde]
impl Trigger for RegexTrigger {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn check(&self, text: &str) -> BoxResult<bool> {
        let re = Regex::new(&self.re)?;
        Ok(re.is_match(text) ^ self.invert)
    }

    fn slice<'a>(&self, text: &'a str) -> BoxResult<&'a str> {
        let re = Regex::new(&self.re)?;
        match re.find(text) {
            Some(ma) => Ok(&text[ma.start()..ma.end()]),
            _ => Ok(&text[0..0])
        }
    }

    fn get_type(&self) -> TriggerType {
        self.trigger_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_match_trigger() {
        let r = RegexTrigger::new("name", "desc", TriggerType::Success, "test", false);

        assert!(r.check("This is a test string").unwrap());
        assert_eq!(r.slice("This is a test string").unwrap(), "test");
    }

    #[test]
    fn it_should_not_match() {
        let r = RegexTrigger::new("name", "desc", TriggerType::Success, "foo", false);

        assert!(!r.check("This is a test string").unwrap());
        assert_eq!(r.slice("This is a test string").unwrap(), "");
    }

    #[test]
    fn it_should_match_inverted_trigger() {
        let r = RegexTrigger::new("name", "desc", TriggerType::Success, "foo", true);

        assert!(r.check("This is a test string").unwrap());
        assert_eq!(r.slice("This is a test string").unwrap(), "");
    }

    #[test]
    fn it_should_not_match_inverted() {
        let r = RegexTrigger::new("name", "desc", TriggerType::Success, "test", true);

        assert!(!r.check("This is a test string").unwrap());
        assert_eq!(r.slice("This is a test string").unwrap(), "test");
    }
}
