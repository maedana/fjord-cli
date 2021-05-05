extern crate open;
extern crate serde_json;

mod util;
use seahorse::Context;
use std::env;
use std::thread;
use std::time::Duration;
use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Row, Table, Tabs},
    Terminal,
};
use util::event::{Event, Events};
use util::stateful_table::StatefulTable;

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
        open::that(&self.url).unwrap();
    }
}

pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub fn new(titles: Vec<&'a str>) -> TabsState {
        TabsState { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

struct App<'a> {
    tabs: TabsState<'a>,
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

    let mut app = App {
        tabs: TabsState::new(vec![
            "Unchecked Reports",
            "Unchecked Products",
            //            "Unassigned Product(0)",
            //            "Assigned Product(0)",
        ]),
    };

    let mut reports: Vec<Report> = vec![];
    let mut report_table = StatefulTable::new();
    let mut unchecked_product_table = StatefulTable::new();

    // Input
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(5)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(f.size());

            let tabs = generate_tabs(&app);
            f.render_widget(tabs, chunks[0]);

            let t1 = generate_report_table(&report_table);
            let t2 = generate_unchecked_product_table(&unchecked_product_table);
            let inner = match app.tabs.index {
                0 => t1,
                1 => t2,
                _ => unreachable!(),
            };
            f.render_stateful_widget(inner, chunks[1], &mut report_table.state);
        })?;

        match events.next()? {
            Event::Input(key) => match key {
                Key::Char('q') => {
                    break;
                }
                Key::Char('j') => {
                    report_table.next();
                }
                Key::Char('k') => {
                    report_table.previous();
                }
                Key::Char('o') => {
                    let selected_index = report_table.state.selected().unwrap();
                    let report = &reports[selected_index];
                    report.open();
                }
                Key::Down => {
                    report_table.next();
                }
                Key::Up => {
                    report_table.previous();
                }
                Key::Right => app.tabs.next(),
                Key::Left => app.tabs.previous(),
                Key::Char('l') => {
                    app.tabs.next();
                }
                Key::Char('h') => {
                    app.tabs.previous();
                }

                _ => {}
            },
            Event::Tick => {
                if report_table.items.is_empty() {
                    reports = Report::fetch();
                    let report_items: Vec<Vec<String>> = reports
                        .iter()
                        .map(|r| {
                            vec![
                                r.title().to_string(),
                                r.reported_on().to_string(),
                                r.login_name().to_string(),
                            ]
                        })
                        .collect();
                    report_table.items = report_items;
                }
            }
        }
    }

    Ok(())
}

fn generate_tabs<'a>(app: &'a App) -> Tabs<'a> {
    let titles = app
        .tabs
        .titles
        .iter()
        .map(|t| Spans::from(vec![Span::styled(*t, Style::default().fg(Color::White))]))
        .collect();
    Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(app.tabs.index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        )
}

fn generate_report_table(table: &StatefulTable) -> Table<'static> {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::White);
    let header_cells = ["Title", "Reported Date", "ID"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = table.items().iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(c.clone()));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });
    Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Unchecked Report"),
        )
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Max(10),
        ])
}

fn generate_unchecked_product_table(table: &StatefulTable) -> Table<'static> {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::White);
    let header_cells = ["Title", "Date", "ID"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = table.items().iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(c.clone()));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });
    Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Unchecked Products"),
        )
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Max(10),
        ])
}
