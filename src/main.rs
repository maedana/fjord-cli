extern crate fjord_cli;
extern crate termion;

use fjord_cli::Report;
use seahorse::{App, Command, Context};
use std::convert::TryInto;
use std::env;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let app = App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage("fjord-cli [name]")
        .command(show_reports_command());

    app.run(args);
}

fn write_alt_screen_msg<W: Write>(screen: &mut W) {
    write!(screen, "{}", termion::clear::All).unwrap();
    for (i, report) in Report::fetch().iter().enumerate() {
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
    write_alt_screen_msg(&mut screen);

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
            _ => {}
        }
        write!(screen, "{}", termion::cursor::Goto(cursor_x, cursor_y)).unwrap();
        screen.flush().unwrap();
    }
    write!(screen, "{}", termion::cursor::Show).unwrap();
}

fn show_reports_command() -> Command {
    Command::new("reports")
        .description("Show unchecked reports.")
        .alias("r")
        .usage("fjord-cli reports(r)")
        .action(show_reports_action)
}
