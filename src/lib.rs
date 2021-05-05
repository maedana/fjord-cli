extern crate open;
extern crate serde_json;

mod util;
use crate::util::event::{Event, Events};
use seahorse::Context;
use std::env;
use std::thread;
use std::time::Duration;
use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Terminal,
};

#[derive(Debug)]
pub struct Report {
    title: String,
    url: String,
    reported_on: String,
    login_name: String,
}

impl Report {
    pub fn fetch() -> Vec<Report> {
        let mut page = 1;
        let mut reports = vec![];
        loop {
            let url = format!(
                "https://bootcamp.fjord.jp/api/reports/unchecked.json?page={}",
                page
            );
            let resp = ureq::get(&url)
                .set("Authorization", &env::var("FJORD_JWT_TOKEN").unwrap())
                .call()
                .unwrap();
            let json: serde_json::Value = resp.into_json().unwrap();
            let report_array = json["reports"].as_array().unwrap();
            if report_array.is_empty() {
                break;
            }
            for r in json["reports"].as_array().unwrap().iter() {
                reports.push(Report {
                    title: r["title"].as_str().unwrap().to_string(),
                    url: r["url"].as_str().unwrap().to_string(),
                    reported_on: r["reportedOn"].as_str().unwrap().to_string(),
                    login_name: r["user"]["login_name"].as_str().unwrap().to_string(),
                })
            }
            page += 1;
            thread::sleep(Duration::from_millis(500));
        }
        reports
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn reported_on(&self) -> &str {
        &self.reported_on
    }

    pub fn login_name(&self) -> &str {
        &self.login_name
    }

    pub fn open(&self) {
        open::that(&self.url);
    }
}

pub struct StatefulTable {
    state: TableState,
    items: Vec<Vec<String>>,
}

impl StatefulTable {
    fn new(reports: &Vec<Report>) -> StatefulTable {
        let items: Vec<Vec<String>> = reports
            .iter()
            .map(|r| {
                vec![
                    r.title().to_string(),
                    r.reported_on().to_string(),
                    r.login_name().to_string(),
                ]
            })
            .collect();
        StatefulTable {
            state: TableState::default(),
            items,
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub fn reports_action(_c: &Context) {
    render_reports().unwrap()
}
fn render_reports() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let reports = Report::fetch();
    let mut table = StatefulTable::new(&reports);

    // Input
    loop {
        terminal.draw(|f| {
            let rects = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(5)
                .split(f.size());

            let selected_style = Style::default().add_modifier(Modifier::REVERSED);
            let normal_style = Style::default().bg(Color::White);
            let header_cells = ["タイトル", "日付", "ID"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black)));
            let header = Row::new(header_cells)
                .style(normal_style)
                .height(1)
                .bottom_margin(1);
            let rows = table.items.iter().map(|item| {
                let height = item
                    .iter()
                    .map(|content| content.chars().filter(|c| *c == '\n').count())
                    .max()
                    .unwrap_or(0)
                    + 1;
                let cells = item.iter().map(|c| Cell::from(c.clone()));
                Row::new(cells).height(height as u16).bottom_margin(1)
            });
            let t = Table::new(rows)
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("未チェック日報"),
                )
                .highlight_style(selected_style)
                .highlight_symbol(">> ")
                .widths(&[
                    Constraint::Percentage(50),
                    Constraint::Length(30),
                    Constraint::Max(10),
                ]);
            f.render_stateful_widget(t, rects[0], &mut table.state);
        })?;

        if let Event::Input(key) = events.next()? {
            match key {
                Key::Char('q') => {
                    break;
                }
                Key::Char('j') => {
                    table.next();
                }
                Key::Char('k') => {
                    table.previous();
                }
                Key::Char('o') => {
                    let selected_index = table.state.selected().unwrap();
                    let report = &reports[selected_index];
                    report.open();
                }
                Key::Down => {
                    table.next();
                }
                Key::Up => {
                    table.previous();
                }
                _ => {}
            }
        };
    }

    Ok(())
}
