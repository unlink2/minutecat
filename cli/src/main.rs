mod app;
pub mod event;
mod tab;

extern crate chrono;
extern crate minutecat;
extern crate termion;
extern crate tokio;
extern crate tui;

use app::App;
use minutecat::interface::command_line;
use std::{error::Error, io};
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let interface = command_line()?;
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    let mut app = App::new(interface, terminal);

    app.init().await?;
    while !app.update().await? {}

    // always save in the end!
    // interface.logset.to_file(&interface.cfg_path)?;

    Ok(())
}
