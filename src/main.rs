#![feature(iterator_try_collect)]
#![feature(exit_status_error)]
#![feature(cfg_match)]

use owo_colors::OwoColorize;

use crate::app::App;
use crate::options::Options;
use crate::spec::Spec;

mod app;
mod builtins;
mod fetcher;
mod glob;
mod installer;
mod options;
mod package;
mod spec;

pub type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

fn main() {
    let options = Options::parse();
    let app = App::new(options);

    if let Err(e) = app.run() {
        eprintln!("{} {:?}", "Error:".red().bold(), e);
        std::process::exit(128);
    }
}
