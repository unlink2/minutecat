use super::serde::{Serialize, Deserialize};
use super::typetag;

/// TriggerType describes if a trigger
/// should be interpreted as an error,
/// a warning or success
#[derive(Serialize, Deserialize)]
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
    fn check(&self, text: &str) -> bool;
    fn get_type(&self) -> TriggerType;
}
