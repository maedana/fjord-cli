extern crate fjord_cli;
extern crate termion;
extern crate ureq;

use anyhow::Result;

fn main() -> Result<()> {
    fjord_cli::render_review_screen()?;
    Ok(())
}
