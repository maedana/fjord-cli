extern crate open;

use seahorse::{Command, Context};
use std::convert::TryInto;
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
        return vec![
            Report {
                title: "Yahoo".to_string(),
                url: "https://yahoo.co.jp".to_string(),
            },
            Report {
                title: "Google".to_string(),
                url: "https://google.co.jp".to_string(),
            },
            Report {
                title: "ブートキャンプ".to_string(),
                url: "https://bootcamp.fjord.jp".to_string(),
            },
        ];
    }

    pub fn screen_label(&self) -> String {
        format!("{} {}", &self.title, &self.url)
    }

    pub fn open(&self) {
        open::that(&self.url);
    }
}

pub fn show_reports_command() -> Command {
    Command::new("reports")
        .description("Show unchecked reports.")
        .alias("r")
        .usage("fjord-cli reports(r)")
        .action(show_reports_action)
}

fn write_alt_screen_msg<W: Write>(screen: &mut W, reports: &Vec<Report>) {
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

fn show_reports_action(_c: &Context) {
    let stdin = stdin();
    let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());
    write!(screen, "{}", termion::cursor::Hide).unwrap();
    let reports = Report::fetch();
    write_alt_screen_msg(&mut screen, &reports);

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
