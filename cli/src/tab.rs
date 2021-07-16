use super::minutecat::logset::LogSet;

pub struct TabManager {
    pub index: usize,
    pub logs: LogSet
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            index: 0,
            logs: LogSet::new()
        }
    }
}
