
pub struct TabManager {
    pub max: usize,
    pub index: usize,
    pub scroll: (u16, u16)
}

impl TabManager {
    pub fn new(max: usize) -> Self {
        Self {
            index: 0,
            max,
            scroll: (0, 0)
        }
    }

    pub fn up(&mut self) {
        if self.scroll.0 < u16::MAX {
            self.scroll.0 += 1;
        }
    }

    pub fn down(&mut self) {
        if self.scroll.0 > 0 {
            self.scroll.0 -= 1;
        }
    }

    pub fn next(&mut self) {
        self.scroll = (0, 0);
        if self.index >= self.max-1 {
            self.index = 0;
        } else {
            self.index += 1;
        }
    }

    pub fn prev(&mut self) {
        self.scroll = (0, 0);
        if self.index <= 0 {
            self.index = self.max-1;
        } else {
            self.index -= 1;
        }
    }
}
