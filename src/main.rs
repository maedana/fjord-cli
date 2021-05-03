extern crate fjord_cli;
extern crate termion;

use seahorse::App;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let app = App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage("fjord-cli [name]")
        .command(fjord_cli::show_reports_command());

    app.run(args);
}
