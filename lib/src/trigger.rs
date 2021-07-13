use super::serde::{Serialize, Deserialize};
use super::typetag;
use super::error::BoxResult;
use super::regex::Regex;

/// TriggerType describes if a trigger
/// should be interpreted as an error,
/// a warning or success
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum TriggerType {
    Success,
    Warning,
    Error
}

/// A trigger is anything that can
/// cause a logfile notification to appear
/// e.g. regex match, time since last change
#[typetag::serde(tag = "type")]
pub trait Trigger {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn check(&self, text: &str) -> BoxResult<bool>;
    fn get_type(&self) -> TriggerType;
}

#[derive(Serialize, Deserialize)]
pub struct RegexTrigger {
    name: String,
    description: String,
    trigger_type: TriggerType,
    re: String
}

impl RegexTrigger {
    pub fn new(name: &str, description: &str, trigger_type: TriggerType, re: &str) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            trigger_type,
            re: re.into()
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
        Ok(re.is_match(text))
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
        let r = RegexTrigger::new("name", "desc", TriggerType::Success, "test");

        assert!(r.check("This is a test string").unwrap());
    }

    #[test]
    fn it_should_not_match() {
        let r = RegexTrigger::new("name", "desc", TriggerType::Success, "foo");

        assert!(!r.check("This is a test string").unwrap());
    }
}
