use super::minutecat::logset::LogSet;

pub struct TabManager {
    pub index: usize,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            index: 0,
        }
    }
}
