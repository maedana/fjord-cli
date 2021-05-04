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

    pub fn screen_label(&self) -> String {
        format!("{}", &self.title)
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

pub struct StatefulTable<'a> {
    state: TableState,
    items: Vec<Vec<&'a str>>,
}

impl<'a> StatefulTable<'a> {
    fn new() -> StatefulTable<'a> {
        StatefulTable {
            state: TableState::default(),
            items: vec![
                vec!["Row11", "Row12", "Row13"],
                vec!["Row21", "Row22", "Row23"],
                vec!["Row31", "Row32", "Row33"],
                vec!["Row41", "Row42", "Row43"],
                vec!["Row51", "Row52", "Row53"],
                vec!["Row61", "Row62\nTest", "Row63"],
                vec!["Row71", "Row72", "Row73"],
                vec!["Row81", "Row82", "Row83"],
                vec!["Row91", "Row92", "Row93"],
                vec!["Row101", "Row102", "Row103"],
                vec!["Row111", "Row112", "Row113"],
                vec!["Row121", "Row122", "Row123"],
                vec!["Row131", "Row132", "Row133"],
                vec!["Row141", "Row142", "Row143"],
                vec!["Row151", "Row152", "Row153"],
                vec!["Row161", "Row162", "Row163"],
                vec!["Row171", "Row172", "Row173"],
                vec!["Row181", "Row182", "Row183"],
                vec!["Row191", "Row192", "Row193"],
            ],
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

pub fn table_action(_c: &Context) {
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
            let normal_style = Style::default().bg(Color::Blue);
            let header_cells = ["Header1", "Header2", "Header3"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
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
                let cells = item.iter().map(|c| Cell::from(*c));
                Row::new(cells).height(height as u16).bottom_margin(1)
            });
            let t = Table::new(rows)
                .header(header)
                .block(Block::default().borders(Borders::ALL).title("Table"))
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
