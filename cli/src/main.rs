use std::io;
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen, event::Event, input::TermRead};
use termion::async_stdin;
use std::io::{Read, Write, stdout};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Tabs},
    Terminal,
};

fn main() -> Result<(), io::Error> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut stdin = async_stdin().bytes();

    let mut i = 0;
    loop {
        i += 1;
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default()
                .title(format!("{}", i))
                .borders(Borders::ALL);
            f.render_widget(block, size);
        })?;

        // input handler for immediate commands

        match stdin.next() {
            Some(Ok(b'q')) => break,
            _ => {}
        }
    }

    Ok(())
}
