use super::tab::TabManager;
use super::minutecat::interface::Interface;


pub struct App {
    pub tabs: TabManager,
    pub interface: Interface
}

impl App {
    pub fn new(interface: Interface) -> Self {
        Self {
            tabs: TabManager::new(interface.logset.len()),
            interface
        }
    }
}
