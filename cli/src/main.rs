pub mod event;
mod app;
mod tab;

extern crate termion;
extern crate tui;
extern crate minutecat;
extern crate chrono;

use app::App;
use event::{Event, Events};
use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Terminal,
};
use chrono::prelude::DateTime;
use chrono::Local;
use minutecat::interface::command_line;
use std::time::{UNIX_EPOCH, Duration};

fn main() -> Result<(), Box<dyn Error>> {
    let interface = command_line()?;
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let mut app = App::new(interface);

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

            let log = &app.interface.logset.logs[app.tabs.index];

            let d = UNIX_EPOCH + Duration::from_millis(log.task.next_time() as u64);
            let datetime = DateTime::<Local>::from(d);

            // render content
            let content = Paragraph::new(log.text.clone())
                .block(Block::default().borders(Borders::ALL).title(
                        format!("{} Next: {}", log.name.clone(), datetime.format("%Y-%m-%d %H:%M:%S"))))
                .wrap(Wrap { trim: true });
            f.render_widget(content, chunks[1]);
        })?;

        // update logs
        for log in &mut app.interface.logset.logs {
            log.update(&mut vec![])?;
        }

        if let Event::Input(input) = events.next()? {
            match input {
                Key::Char('q') => {
                    break;
                },
                Key::Right => app.tabs.next(),
                Key::Left => app.tabs.prev(),
                _ => {}
            }
        }
    }

    // always save in the end!
    app.interface.logset.to_file(&app.interface.cfg_path)?;

    Ok(())
}
