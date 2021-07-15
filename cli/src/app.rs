use super::tab::TabManager;
use std::{error::Error, io};
use super::termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use super::tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Tabs},
    Terminal,
};

pub struct App {
    tabs: TabManager
}
