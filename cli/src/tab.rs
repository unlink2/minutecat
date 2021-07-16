
pub struct TabManager {
    pub max: usize,
    pub index: usize,
}

impl TabManager {
    pub fn new(max: usize) -> Self {
        Self {
            index: 0,
            max
        }
    }

    pub fn next(&mut self) {
        if self.index >= self.max-1 {
            self.index = 0;
        } else {
            self.index += 1;
        }
    }

    pub fn prev(&mut self) {
        if self.index <= 0 {
            self.index = self.max-1;
        } else {
            self.index -= 1;
        }
    }
}
