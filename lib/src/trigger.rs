use super::serde::{Serialize, Deserialize};
use super::typetag;

#[typetag::serde(tag = "type")]
pub trait Trigger {

}
