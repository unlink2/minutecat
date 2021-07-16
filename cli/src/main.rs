mod event;
mod app;
mod tab;

extern crate termion;
extern crate tui;
extern crate minutecat;

use app::App;
use event::{Event, Events};
use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen, input::TermRead};
use std::io::{Read, Write, stdout};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Tabs},
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
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let app = App::new(interface);

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3),Constraint::Min(0)].as_ref())
                .split(size);

            // outside border
            let block = Block::default()
                .borders(Borders::ALL);
            f.render_widget(block, size);

            // get tab titles
            let titles: Vec<Spans> = app
                .interface.logset.logs
                .iter()
                .map(|t| {
                    let (first, rest) = t.name.split_at(1);
                    Spans::from(vec![
                        Span::styled(first, Style::default().fg(Color::Yellow)),
                        Span::styled(rest, Style::default().fg(Color::Green)),
                    ])
                })
                .collect();

            // render tabs
            let tabs = Tabs::new(titles)
                .block(Block::default().borders(Borders::ALL).title("Tabs"))
                .select(app.tabs.index)
                .style(Style::default().fg(Color::Cyan))
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Black),
                );
            f.render_widget(tabs, chunks[0]);
        })?;

        if let Event::Input(input) = events.next()? {
            match input {
                Key::Char('q') => {
                    break;
                }
                _ => {}
            }
        }
    }

    // always save in the end!
    app.interface.logset.to_file(&app.interface.cfg_path)?;

    Ok(())
}
