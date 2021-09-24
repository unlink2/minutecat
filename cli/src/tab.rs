use super::minutecat::logfile::{Event, EventHandler};
use super::minutecat::task::TimeMs;
use super::minutecat::trigger::TriggerType;
use std::collections::HashMap;

#[derive(Clone)]
pub struct TabState {
    pub trigger_type: TriggerType,
    pub slices: HashMap<String, String>,
    pub text: String,
    pub name: String,
    pub next_time: TimeMs,
}

impl TabState {
    pub fn new() -> Self {
        Self {
            text: "".into(),
            name: "".into(),
            trigger_type: TriggerType::NoEvent,
            slices: HashMap::new(),
            next_time: 0,
        }
    }
}

pub struct TabManager {
    pub state: Vec<TabState>,
    pub max: usize,
    pub index: usize,
    pub scroll: (u16, u16),
    pub tab_offset: usize,
}

impl TabManager {
    pub fn new(max: usize) -> Self {
        Self {
            state: vec![TabState::new(); max],
            index: 0,
            max,
            scroll: (0, 0),
            tab_offset: 0,
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
        if self.index >= self.max - 1 {
            self.index = 0;
        } else {
            self.index += 1;
        }
    }

    pub fn prev(&mut self) {
        self.scroll = (0, 0);
        if self.index == 0 {
            self.index = self.max - 1;
        } else {
            self.index -= 1;
        }
    }

    pub fn next_offset(&mut self) {
        if self.tab_offset >= self.max - 1 {
            self.tab_offset = 0;
        } else {
            self.tab_offset += 1;
        }
    }

    pub fn prev_offset(&mut self) {
        if self.tab_offset == 0 {
            self.tab_offset = self.max - 1;
        } else {
            self.tab_offset -= 1;
        }
    }
}

impl EventHandler for TabState {
    fn on_event(&mut self, event: &Event) {
        if let Some(trigger) = event.trigger {
            if event.did_trigger {
                self.slices.insert(
                    trigger.name().into(),
                    trigger.slice(event.text).unwrap_or("").into(),
                );
                self.trigger_type = trigger.get_type();
            } else if self.slices.contains_key(trigger.name()) {
                self.slices.remove(trigger.name());
            }
        }
        if event.did_trigger || self.name.is_empty() {
            self.text = event.text.into();
            self.name = event.name.into();
        }
        self.next_time = event.task.next_time();
    }
}
