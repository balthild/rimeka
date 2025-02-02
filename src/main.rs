#![feature(try_blocks)]
#![feature(iterator_try_collect)]

use owo_colors::OwoColorize;

use crate::app::App;
use crate::options::Options;
use crate::spec::Spec;

mod app;
mod builtins;
mod installer;
mod options;
mod package;
mod regex;
mod spec;

pub type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

fn main() {
    let options = Options::parse();
    let app = App::new(options);

    if let Err(e) = app.run() {
        eprintln!("{} {:?}", "Error:".red().bold(), e);
        std::process::exit(99);
    }
}
