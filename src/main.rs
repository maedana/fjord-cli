extern crate fjord_cli;
extern crate termion;
extern crate ureq;

use seahorse::{App, Command};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let app = App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage("fjord-cli [name]")
        .command(reports_command());

    app.run(args);
}

fn reports_command() -> Command {
    Command::new("review")
        .description("Open unchecked reports and products.")
        .alias("r")
        .usage("fjord-cli review(r)")
        .action(fjord_cli::review_action)
}
