extern crate open;
extern crate serde_json;

mod util;
use crate::util::event::{Event, Events};
use seahorse::Context;
use std::env;
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
}

impl Report {
    pub fn fetch() -> Vec<Report> {
        let url = "http://localhost:3000/api/reports/unchecked.json";
        let resp = ureq::get(url)
            .set("Authorization", &env::var("FJORD_JWT_TOKEN").unwrap())
            .call()
            .unwrap();
        let json: serde_json::Value = resp.into_json().unwrap();
        json["reports"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| Report {
                title: r["title"].as_str().unwrap().to_string(),
                url: r["url"].as_str().unwrap().to_string(),
            })
            .collect()
    }

    pub fn screen_label(&self) -> &str {
        &self.title
    }

    pub fn open(&self) {
        open::that(&self.url);
    }
}

// screenをwrapするような構造体作るとすっきりしそう
// struct ReportScreen {
//    screen: AlternateScreen<RawTerminal>
//    x: u16
//    y: u16
//    reports: Vec<Report>
//    current_report: Option<Report>
// }
// みたいな感じ?
//pub fn reports_action(_c: &Context) {
//    let stdin = stdin();
//    let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());
//    write!(screen, "{}", termion::cursor::Hide).unwrap();
//    let reports = Report::fetch();
//    write_reports(&mut screen, &reports);
//
//    // カーソルを最初の位置へセット
//    let cursor_x = 1;
//    let mut cursor_y = 1;
//    write!(
//        screen,
//        "{}{}",
//        termion::cursor::Goto(cursor_x, cursor_y),
//        termion::cursor::Show
//    )
//    .unwrap();
//
//    screen.flush().unwrap();
//
//    for c in stdin.keys() {
//        match c.unwrap() {
//            Key::Char('q') => break,
//            Key::Ctrl('c') => break,
//            Key::Char('j') => {
//                cursor_y += 1;
//            }
//            Key::Char('k') => {
//                if cursor_y > 1 {
//                    cursor_y -= 1;
//                }
//            }
//            Key::Char('o') => {
//                let current_index: usize = From::from(cursor_y - 1);
//                let report = &reports[current_index];
//                report.open();
//            }
//            _ => {}
//        }
//        write!(screen, "{}", termion::cursor::Goto(cursor_x, cursor_y)).unwrap();
//        screen.flush().unwrap();
//    }
//    write!(screen, "{}", termion::cursor::Show).unwrap();
//}
//
//fn write_reports<W: Write>(screen: &mut W, reports: &Vec<Report>) {
//    write!(screen, "{}", termion::clear::All).unwrap();
//    for (i, report) in reports.iter().enumerate() {
//        write!(
//            screen,
//            "{}{}",
//            termion::cursor::Goto(1, (i + 1).try_into().unwrap()),
//            report.screen_label()
//        )
//        .unwrap();
//    }
//}

pub struct StatefulTable {
    state: TableState,
    items: Vec<Vec<String>>,
}

impl StatefulTable {
    fn new() -> StatefulTable {
        let items: Vec<Vec<String>> = Report::fetch()
            .iter()
            .map(|r| {
                vec![
                    r.screen_label().to_string(),
                    "yyyy-mm-dd".to_string(),
                    "xxxxxxxx".to_string(),
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
    render().unwrap()
}
fn render() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let mut table = StatefulTable::new();

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
