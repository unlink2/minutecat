use super::event::{Event, Events};
use super::minutecat::error::Error;
use super::minutecat::interface::Interface;
use super::minutecat::trigger::TriggerType;
use super::tab::TabManager;
use chrono::prelude::DateTime;
use chrono::Local;
use std::time::{Duration, UNIX_EPOCH};
use termion::event::Key;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};

pub struct App<B>
where
    B: Backend,
{
    pub tabs: TabManager,
    pub interface: Interface,
    terminal: Terminal<B>,
    events: Events,
}

impl<B> App<B>
where
    B: Backend,
{
    pub fn new(interface: Interface, terminal: Terminal<B>) -> Self {
        Self {
            tabs: TabManager::new(interface.logset.len()),
            interface,
            terminal,
            events: Events::new(),
        }
    }

    pub async fn update_logs(interface: &mut Interface, tabs: &mut TabManager, force: bool) {
        for (index, log) in &mut interface.logset.logs.iter_mut().enumerate() {
            // update logs
            // TODO handle errors better!

            let res = if force {
                log.force_update(&mut vec![&mut tabs.state[index]]).await
            } else {
                log.update(&mut vec![&mut tabs.state[index]]).await
            };
            match res {
                Ok(_) => {}
                Err(err) => {
                    tabs.state[index]
                        .slices
                        .insert("Error".into(), format!("{}", err));
                }
            }
        }
    }

    pub async fn init(&mut self) -> Result<(), Error> {
        Self::update_logs(&mut self.interface, &mut self.tabs, true).await;

        Ok(())
    }

    pub async fn update(&mut self) -> Result<bool, Error> {
        // do forever
        Self::update_logs(&mut self.interface, &mut self.tabs, false).await;

        self.render()?;
        let next_event = match self.events.next() {
            Ok(next_event) => next_event,
            Err(_err) => return Err(Error::GenericError),
        };

        if let Event::Input(input) = next_event {
            match input {
                Key::Char('q') => {
                    return Ok(true);
                }
                Key::Right => self.tabs.next(),
                Key::Left => self.tabs.prev(),
                Key::Up => self.tabs.down(),
                Key::Down => self.tabs.up(),
                Key::PageUp => self.tabs.next_offset(),
                Key::PageDown => self.tabs.prev_offset(),
                _ => {}
            }
        }

        return Ok(false);
    }

    pub fn render_tabs(f: &mut Frame<B>, tab_manager: &TabManager, chunk: &Rect) {
        // get tab titles
        let titles: Vec<Spans> = tab_manager.state[tab_manager.tab_offset..]
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let tab = &tab_manager.state[tab_manager.tab_offset + i];

                let color = match tab.trigger_type {
                    TriggerType::Success => Color::Green,
                    TriggerType::Warning => Color::Yellow,
                    TriggerType::Error => Color::Red,
                    _ => Color::White,
                };
                Spans::from(vec![Span::styled(&t.name, Style::default().fg(color))])
            })
            .collect();

        // render tabs
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title(format!(
                "Tabs {}/{}",
                tab_manager.index + 1,
                tab_manager.max
            )))
            .select(tab_manager.index)
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Black),
            );
        f.render_widget(tabs, *chunk);
    }

    pub fn render_no_logs(f: &mut Frame<B>, tab_manager: &TabManager, chunk: &Rect) {
        let content = Paragraph::new("No Logs")
            .block(Block::default().borders(Borders::ALL).title("No Logs"))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .scroll(tab_manager.scroll);
        f.render_widget(content, *chunk);
    }

    pub fn render_content(f: &mut Frame<B>, tab_manager: &TabManager, chunk: &Rect) {
        let log = &tab_manager.state[tab_manager.index];

        let d = UNIX_EPOCH + Duration::from_millis(log.next_time as u64);
        let datetime = DateTime::<Local>::from(d);

        // render content
        let content = Paragraph::new(log.text.clone())
            .block(Block::default().borders(Borders::ALL).title(format!(
                "{} Next: {}",
                log.name.clone(),
                datetime.format("%Y-%m-%d %H:%M:%S")
            )))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .scroll(tab_manager.scroll);
        f.render_widget(content, *chunk);
    }

    pub fn render_info(f: &mut Frame<B>, tab_manager: &TabManager, chunk: &Rect) {
        // get slices
        let tab = &tab_manager.state[tab_manager.index];

        let titles: Vec<Spans> = tab
            .slices
            .iter()
            .enumerate()
            .map(|(_i, t)| {
                let color = match tab.trigger_type {
                    TriggerType::Success => Color::Green,
                    TriggerType::Warning => Color::Yellow,
                    TriggerType::Error => Color::Red,
                    _ => Color::White,
                };
                Spans::from(vec![Span::styled(
                    format!("{}={}", t.0, t.1),
                    Style::default().fg(color),
                )])
            })
            .collect();

        // render tabs
        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Info ({})", tab.slices.len())),
            )
            .select(tab_manager.index)
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Black),
            );
        f.render_widget(tabs, *chunk);
    }

    pub fn render(&mut self) -> Result<(), Error> {
        let tab_manager = &mut self.tabs;

        self.terminal.draw(|mut f| {
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(0),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            // outside border
            let block = Block::default().borders(Borders::NONE);
            f.render_widget(block, size);

            Self::render_tabs(&mut f, &tab_manager, &chunks[0]);

            if tab_manager.state.len() == 0 {
                Self::render_no_logs(&mut f, &tab_manager, &chunks[1]);
            } else {
                Self::render_content(&mut f, &tab_manager, &chunks[1]);
                // render info about matches
                Self::render_info(&mut f, &tab_manager, &chunks[2]);
            }
        })?;
        Ok(())
    }
}
