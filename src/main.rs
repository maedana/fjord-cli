use seahorse::{App, Command, Context};
use std::env;

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

fn show_reports_action(_c: &Context) {
    println!("show reports!!!");
}

fn show_reports_command() -> Command {
    Command::new("reports")
        .description("show unchecked reports")
        .alias("r")
        .usage("fjord-cli reports(r)")
        .action(show_reports_action)
}
