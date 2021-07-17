pub mod event;
mod app;
mod tab;

extern crate termion;
extern crate tui;
extern crate minutecat;
extern crate chrono;

use app::App;
use std::{error::Error, io};
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    Terminal,
};
use minutecat::interface::command_line;

fn main() -> Result<(), Box<dyn Error>> {
    let interface = command_line()?;
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let terminal = Terminal::new(backend)?;


    let mut app = App::new(interface, terminal);

    app.init()?;
    while !app.update()? {
    }

    // always save in the end!
    app.interface.logset.to_file(&app.interface.cfg_path)?;

    Ok(())
}
