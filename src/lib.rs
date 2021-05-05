extern crate open;
extern crate serde_json;

mod models;
mod util;

use models::product::Product;
use models::report::Report;
use seahorse::Context;
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
impl<'a> App<'a> {
    // MEMO Appのメソッドに出来るんじゃないか
    fn generate_tabs(&'a self) -> Tabs<'a> {
        let titles = self
            .tabs
            .titles
            .iter()
            .map(|t| Spans::from(vec![Span::styled(*t, Style::default().fg(Color::White))]))
            .collect();
        Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .select(self.tabs.index)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Black),
            )
    }
}

pub fn review_action(_c: &Context) {
    render_review_screen().unwrap()
}

fn render_review_screen() -> Result<(), Box<dyn Error>> {
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
    let mut unchecked_products: Vec<Product> = vec![];
    let mut unchecked_product_table = StatefulTable::new();

    // Input
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(5)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(f.size());

            let tabs = app.generate_tabs();
            f.render_widget(tabs, chunks[0]);

            let t1 = generate_table_widget(&report_table)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Unchecked Report"),
                )
                .header(generate_header(vec!["Title", "Reported Date", "ID"]))
                .widths(&[
                    Constraint::Percentage(50),
                    Constraint::Length(30),
                    Constraint::Max(10),
                ]);
            let t2 = generate_table_widget(&unchecked_product_table)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Unchecked Products"),
                )
                .header(generate_header(vec!["Title", "Date", "ID"]))
                .widths(&[
                    Constraint::Percentage(50),
                    Constraint::Length(30),
                    Constraint::Max(10),
                ]);
            // TODO: タブのindexがマジックナンバーなのでリーダブルにしたい
            let inner = match app.tabs.index {
                0 => t1,
                1 => t2,
                _ => unreachable!(),
            };
            let state = match app.tabs.index {
                0 => &mut report_table.state,
                1 => &mut unchecked_product_table.state,
                _ => unreachable!(),
            };
            f.render_stateful_widget(inner, chunks[1], state);
        })?;

        let current_table = match app.tabs.index {
            0 => &mut report_table,
            1 => &mut unchecked_product_table,
            _ => unreachable!(),
        };

        match events.next()? {
            Event::Input(key) => match key {
                Key::Char('q') => {
                    break;
                }
                Key::Char('o') => {
                    match app.tabs.index {
                        0 => {
                            let selected_index = report_table.state.selected().unwrap();
                            let report = &reports[selected_index];
                            report.open();
                        }
                        1 => {
                            let selected_index = unchecked_product_table.state.selected().unwrap();
                            let product = &unchecked_products[selected_index];
                            product.open();
                        }
                        _ => unreachable!(),
                    };
                }
                Key::Char('j') => {
                    current_table.next();
                }
                Key::Char('k') => {
                    current_table.previous();
                }
                Key::Down => {
                    current_table.next();
                }
                Key::Up => {
                    current_table.previous();
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
                if unchecked_product_table.items.is_empty() {
                    unchecked_products = Product::fetch();
                    let unchecked_product_items: Vec<Vec<String>> = unchecked_products
                        .iter()
                        .map(|p| {
                            vec![
                                p.title().to_string(),
                                p.updated_on().to_string(),
                                p.login_name().to_string(),
                            ]
                        })
                        .collect();
                    unchecked_product_table.items = unchecked_product_items;
                }
            }
        }
    }

    Ok(())
}

fn generate_table_widget<'a>(table: &StatefulTable) -> Table<'a> {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
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
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
}

fn generate_header(headers: Vec<&str>) -> Row {
    let normal_style = Style::default().bg(Color::White);
    let header_cells = headers
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black)));
    Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1)
}
