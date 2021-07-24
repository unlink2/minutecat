use super::tab::TabManager;
use super::event::{Events, Event};
use super::minutecat::interface::Interface;
use super::minutecat::error::BoxResult;
use super::minutecat::trigger::TriggerType;
use termion::event::Key;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Terminal,
    Frame
};
use chrono::prelude::DateTime;
use chrono::Local;
use std::time::{UNIX_EPOCH, Duration};


pub struct App<B>
where B: Backend {
    pub tabs: TabManager,
    pub interface: Interface,
    terminal: Terminal<B>,
    events: Events
}

impl<B> App<B>
where B: Backend {
    pub fn new(interface: Interface, terminal: Terminal<B>) -> Self {
        Self {
            tabs: TabManager::new(interface.logset.len()),
            interface,
            terminal,
            events: Events::new()
        }
    }

    pub fn init(&mut self) -> BoxResult<()> {
        let interface = &mut self.interface;
        let tabs = &mut self.tabs;
        for (i, log) in interface.logset.logs.iter_mut().enumerate() {
            log.force_update(&mut vec![&mut tabs.state[i]])?;
        }
        Ok(())
    }

    pub fn update(&mut self) -> BoxResult<bool> {
        self.render()?;

        let interface = &mut self.interface;
        let tabs = &mut self.tabs;
        // update logs

        for (i, log) in interface.logset.logs.iter_mut().enumerate() {
            log.update(&mut vec![&mut tabs.state[i]])?;
        }

        if let Event::Input(input) = self.events.next()? {
            match input {
                Key::Char('q') => {
                    return Ok(true);
                },
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

    pub fn render_tabs(f: &mut Frame<B>, interface: &Interface, tab_manager: &TabManager, chunk: &Rect) {
        // get tab titles
        let titles: Vec<Spans> = interface
            .logset.logs[tab_manager.tab_offset..]
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let tab = &tab_manager.state[tab_manager.tab_offset+i];

                let color = match tab.trigger_type {
                    TriggerType::Success => Color::Green,
                    TriggerType::Warning => Color::Yellow,
                    TriggerType::Error => Color::Red,
                    _ => Color::White
                };
                Spans::from(vec![
                    Span::styled(&t.name, Style::default().fg(color)),
                ])
            })
        .collect();

        // render tabs
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title(
                    format!("Tabs {}/{}", tab_manager.index+1, tab_manager.max)))
            .select(tab_manager.index)
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(
                Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
            );
        f.render_widget(tabs, *chunk);
    }

    pub fn render_no_logs(f: &mut Frame<B>, _interface: &Interface, tab_manager: &TabManager, chunk: &Rect) {
        let content = Paragraph::new("No Logs")
            .block(Block::default().borders(Borders::ALL).title(
                    "No Logs"))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .scroll(tab_manager.scroll);
        f.render_widget(content, *chunk);
    }

    pub fn render_content(f: &mut Frame<B>, interface: &Interface, tab_manager: &TabManager, chunk: &Rect) {
        let log = &interface.logset.logs[tab_manager.index];

        let d = UNIX_EPOCH + Duration::from_millis(log.task.next_time() as u64);
        let datetime = DateTime::<Local>::from(d);

        // render content
        let content = Paragraph::new(log.text.clone())
            .block(Block::default().borders(Borders::ALL).title(
                    format!("{} Next: {}", log.name.clone(), datetime.format("%Y-%m-%d %H:%M:%S"))))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .scroll(tab_manager.scroll);
        f.render_widget(content, *chunk);
    }

    pub fn render_info(f: &mut Frame<B>, _interface: &Interface, tab_manager: &TabManager, chunk: &Rect) {
        // get slices
        let tab = &tab_manager.state[tab_manager.index];

        let titles: Vec<Spans> = tab.slices
            .iter()
            .enumerate()
            .map(|(_i, t)| {
                let color = match tab.trigger_type {
                    TriggerType::Success => Color::Green,
                    TriggerType::Warning => Color::Yellow,
                    TriggerType::Error => Color::Red,
                    _ => Color::White
                };
                Spans::from(vec![
                    Span::styled(format!("{}={}", t.0, t.1), Style::default().fg(color)),
                ])
            })
        .collect();

        // render tabs
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title(format!("Info ({})", tab.slices.len())))
            .select(tab_manager.index)
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(
                Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
            );
        f.render_widget(tabs, *chunk);
    }

    pub fn render(&mut self) -> BoxResult<()> {
        let interface = &self.interface;
        let tab_manager = &self.tabs;

        self.terminal.draw(|mut f| {
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)].as_ref())
                .split(size);

            // outside border
            let block = Block::default()
                .borders(Borders::NONE);
            f.render_widget(block, size);

            Self::render_tabs(&mut f, interface, tab_manager, &chunks[0]);

            if interface.logset.len() == 0 {
                Self::render_no_logs(&mut f, interface, tab_manager, &chunks[1]);
            } else {
                Self::render_content(&mut f, interface, tab_manager, &chunks[1]);
                // render info about matches
                Self::render_info(&mut f, interface, tab_manager, &chunks[2]);
            }
        })?;

        Ok(())
    }
}

