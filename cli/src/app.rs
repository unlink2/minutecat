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
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

pub struct App<B>
where B: Backend {
    pub tabs: Arc<Mutex<TabManager>>,
    pub interface: Arc<Mutex<Interface>>,
    // this thread runs updates
    terminal: Terminal<B>,
    update_handle: Option<thread::JoinHandle<()>>,
    events: Events
}

impl<B> App<B>
where B: Backend {
    pub fn new(interface: Interface, terminal: Terminal<B>) -> Self {
        Self {
            tabs: Arc::new(Mutex::new(TabManager::new(interface.logset.len()))),
            interface: Arc::new(Mutex::new(interface)),
            terminal,
            update_handle: None,
            events: Events::new()
        }
    }

    pub fn update_logs(interface: &Arc<Mutex<Interface>>, tabs: &Arc<Mutex<TabManager>>, index: &mut usize, force: bool) {
        if let (Ok(mut interface), Ok(mut tabs)) = (interface.lock(), tabs.lock()) {
            if interface.logset.len() > 0 {
                // update logs
                // TODO handle errors better!
                let log = &mut interface.logset.logs[*index];

                let res = if force {
                    log.force_update(&mut vec![&mut tabs.state[*index]])
                } else {
                    log.update(&mut vec![&mut tabs.state[*index]])
                };
                match res {
                    Ok(_) => {},
                    Err(err) => { tabs.state[*index].slices.insert("Error".into(), format!("{}", err)); }
                }

                *index += 1;
                if *index >= interface.logset.len() {
                    *index = 0;
                }
            }
        }

        if !force {
            thread::sleep(time::Duration::from_millis(500));
        }
    }

    pub fn init(&mut self) -> BoxResult<()> {
        let interface = self.interface.clone();
        let tabs = self.tabs.clone();

        self.update_handle = Some(thread::spawn(move || {
            // spawn a thread that keeps going forever
            loop {
                let mut index = 0;
                // TODO this migth deadlock!
                loop {
                    Self::update_logs(&interface, &tabs, &mut index, true);
                    if index == 0 {
                        break;
                    }
                }
                break;
            }

            let mut index = 0;
            // do forever
            loop {
                Self::update_logs(&interface, &tabs, &mut index, false);
            }
        }));
        Ok(())
    }

    pub fn update(&mut self) -> BoxResult<bool> {
        self.render()?;

        let tabs = self.tabs.clone();

        if let Ok(mut tabs) = tabs.lock() {
            if let Event::Input(input) = self.events.next()? {
                match input {
                    Key::Char('q') => {
                        return Ok(true);
                    },
                    Key::Right => tabs.next(),
                    Key::Left => tabs.prev(),
                    Key::Up => tabs.down(),
                    Key::Down => tabs.up(),
                    Key::PageUp => tabs.next_offset(),
                    Key::PageDown => tabs.prev_offset(),
                    _ => {}
                }
            }
        }

        thread::sleep(Duration::from_millis(10));

        return Ok(false);
    }

    pub fn render_tabs(f: &mut Frame<B>, tab_manager: &TabManager, chunk: &Rect) {
        // get tab titles
        let titles: Vec<Spans> = tab_manager.state[tab_manager.tab_offset..]
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

    pub fn render_no_logs(f: &mut Frame<B>, tab_manager: &TabManager, chunk: &Rect) {
        let content = Paragraph::new("No Logs")
            .block(Block::default().borders(Borders::ALL).title(
                    "No Logs"))
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
            .block(Block::default().borders(Borders::ALL).title(
                    format!("{} Next: {}", log.name.clone(), datetime.format("%Y-%m-%d %H:%M:%S"))))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .scroll(tab_manager.scroll);
        f.render_widget(content, *chunk);
    }

    pub fn render_info(f: &mut Frame<B>, tab_manager: &TabManager, chunk: &Rect) {
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
        let tab_manager = self.tabs.clone();

        if let Ok(tab_manager) = tab_manager.lock() {
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

                Self::render_tabs(&mut f, &tab_manager, &chunks[0]);

                if tab_manager.state.len() == 0 {
                    Self::render_no_logs(&mut f, &tab_manager, &chunks[1]);
                } else {
                    Self::render_content(&mut f, &tab_manager, &chunks[1]);
                    // render info about matches
                    Self::render_info(&mut f, &tab_manager, &chunks[2]);
                }
            })?;
        }
        Ok(())
    }
}

