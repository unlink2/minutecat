use super::serde::{Serialize, Deserialize};
use super::typetag;


#[typetag::serde(tag = "type")]
pub trait TimeSource {
    fn get_time_ms(&self) -> i128;
}

#[derive(Serialize, Deserialize)]
pub struct Task {
    repeat: bool,
    delay: u64,
    start: i128,
    time_src: Box<dyn TimeSource>
}
