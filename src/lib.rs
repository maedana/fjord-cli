extern crate open;
extern crate serde_json;

use seahorse::Context;
use std::convert::TryInto;
use std::env;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::*;

#[derive(Debug)]
pub struct Report {
    title: String,
    url: String,
}

impl Report {
    pub fn fetch() -> Vec<Report> {
        let url = "https://bootcamp.fjord.jp/api/reports/unchecked.json";
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
pub fn reports_action(_c: &Context) {
    let stdin = stdin();
    let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());
    write!(screen, "{}", termion::cursor::Hide).unwrap();
    let reports = Report::fetch();
    write_reports(&mut screen, &reports);

    // カーソルを最初の位置へセット
    let cursor_x = 1;
    let mut cursor_y = 1;
    write!(
        screen,
        "{}{}",
        termion::cursor::Goto(cursor_x, cursor_y),
        termion::cursor::Show
    )
    .unwrap();

    screen.flush().unwrap();

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('q') => break,
            Key::Ctrl('c') => break,
            Key::Char('j') => {
                cursor_y += 1;
            }
            Key::Char('k') => {
                if cursor_y > 1 {
                    cursor_y -= 1;
                }
            }
            Key::Char('o') => {
                let current_index: usize = From::from(cursor_y - 1);
                let report = &reports[current_index];
                report.open();
            }
            _ => {}
        }
        write!(screen, "{}", termion::cursor::Goto(cursor_x, cursor_y)).unwrap();
        screen.flush().unwrap();
    }
    write!(screen, "{}", termion::cursor::Show).unwrap();
}

fn write_reports<W: Write>(screen: &mut W, reports: &Vec<Report>) {
    write!(screen, "{}", termion::clear::All).unwrap();
    for (i, report) in reports.iter().enumerate() {
        write!(
            screen,
            "{}{}",
            termion::cursor::Goto(1, (i + 1).try_into().unwrap()),
            report.screen_label()
        )
        .unwrap();
    }
}
