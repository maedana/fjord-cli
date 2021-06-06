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

pub enum TabPage {
    UncheckedReports,
    UncheckedProducts,
    UnassignedProducts,
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

    pub fn page(&self) -> TabPage {
        match self.index {
            0 => TabPage::UncheckedReports,
            1 => TabPage::UncheckedProducts,
            2 => TabPage::UnassignedProducts,
            _ => unreachable!(),
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
            "Unassigned Product",
            //            "Assigned Product(0)",
        ]),
    };

    let mut unchecked_reports: Vec<Report> = vec![];
    let mut report_table = StatefulTable::new();
    let mut unchecked_products: Vec<Product> = vec![];
    let mut unchecked_product_table = StatefulTable::new();
    let mut unassigned_products: Vec<Product> = vec![];
    let mut unassigned_product_table = StatefulTable::new();

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

            let inner = match app.tabs.page() {
                TabPage::UncheckedReports => report_table_widget(&report_table.items).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Unchecked Report({})", unchecked_reports.len())),
                ),
                TabPage::UncheckedProducts => product_table_widget(&unchecked_product_table.items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!("Unchecked Products({})", unchecked_products.len())),
                    ),
                TabPage::UnassignedProducts => product_table_widget(
                    &unassigned_product_table.items,
                )
                .block(Block::default().borders(Borders::ALL).title(format!(
                    "Unassigned Products({})",
                    unassigned_products.len()
                ))),
            };
            let state = match app.tabs.page() {
                TabPage::UncheckedReports => &mut report_table.state,
                TabPage::UncheckedProducts => &mut unchecked_product_table.state,
                TabPage::UnassignedProducts => &mut unassigned_product_table.state,
            };
            f.render_stateful_widget(inner, chunks[1], state);
        })?;

        let current_table = match app.tabs.page() {
            TabPage::UncheckedReports => &mut report_table,
            TabPage::UncheckedProducts => &mut unchecked_product_table,
            TabPage::UnassignedProducts => &mut unassigned_product_table,
        };

        match events.next()? {
            Event::Input(key) => match key {
                Key::Char('q') => {
                    break;
                }
                Key::Char('o') => {
                    match app.tabs.page() {
                        TabPage::UncheckedReports => {
                            let selected_index = report_table.state.selected().unwrap_or(0);
                            let report = &unchecked_reports[selected_index];
                            report.open();
                        }
                        TabPage::UncheckedProducts => {
                            let selected_index =
                                unchecked_product_table.state.selected().unwrap_or(0);
                            let product = &unchecked_products[selected_index];
                            product.open();
                        }
                        TabPage::UnassignedProducts => {
                            let selected_index =
                                unassigned_product_table.state.selected().unwrap_or(0);
                            let product = &unassigned_products[selected_index];
                            product.open();
                        }
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
                match app.tabs.page() {
                    TabPage::UncheckedReports => {
                        if unchecked_reports.is_empty() {
                            unchecked_reports = Report::fetch();
                            let report_items: Vec<Vec<String>> = unchecked_reports
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
                    TabPage::UncheckedProducts => {
                        if unchecked_products.is_empty() {
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
                    TabPage::UnassignedProducts => {
                        if unchecked_products.is_empty() {
                            unchecked_products = Product::fetch();
                        }
                        if unassigned_products.is_empty() {
                            unassigned_products = unchecked_products
                                .iter()
                                .cloned()
                                .filter(|p| !p.assigned())
                                .collect();
                            let unassigned_product_items: Vec<Vec<String>> = unchecked_products
                                .iter()
                                .filter(|p| !p.assigned())
                                .map(|p| {
                                    vec![
                                        p.title().to_string(),
                                        p.updated_on().to_string(),
                                        p.login_name().to_string(),
                                    ]
                                })
                                .collect();
                            unassigned_product_table.items = unassigned_product_items;
                        }
                    }
                };
            }
        }
    }

    Ok(())
}

fn report_table_widget(items: &Vec<Vec<String>>) -> Table {
    generate_table_widget(items)
        .header(generate_header(vec!["Title", "Reported Date", "ID"]))
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Max(10),
        ])
}

fn product_table_widget(items: &Vec<Vec<String>>) -> Table {
    generate_table_widget(items)
        .header(generate_header(vec!["Title", "Date", "ID"]))
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Max(10),
        ])
}

fn generate_table_widget<'a>(items: &Vec<Vec<String>>) -> Table<'a> {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let rows = items.iter().map(|item| {
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
