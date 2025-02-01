#![feature(try_blocks)]
#![feature(iterator_try_collect)]
#![feature(associated_type_defaults)]

use std::path::PathBuf;

use anyhow::Context;

use crate::options::{Frontend, Options};
use crate::package::Package;

mod options;
mod package;
mod regex;

pub type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

fn main() {
    let options = Options::parse();
    let app = App { options };

    if let Err(e) = app.run() {
        eprintln!("Error: {:?}", e);
        std::process::exit(99);
    }
}

struct App {
    options: Options,
}

impl App {
    fn run(self) -> Result {
        self.mkdir()?;

        let mut packages = self.resolve_packages()?;

        if self.options.select {
            packages = self.select(packages);
        }

        self.install(packages)
    }

    fn mkdir(&self) -> Result {
        std::fs::create_dir_all(self.data_dir()?)?;
        std::fs::create_dir_all(self.rime_dir()?)?;
        Ok(())
    }

    fn resolve_packages(&self) -> Result<Vec<Package>> {
        let resolved = self
            .options
            .targets
            .iter()
            .map(|x| Package::resolve(x))
            .try_collect::<Vec<_>>()?;

        Ok(resolved.concat())
    }

    fn select(&self, candidates: Vec<Package>) -> Vec<Package> {
        candidates
    }

    fn install(&self, packages: Vec<Package>) -> Result {
        let base = self.repos_dir()?;
        let sources = packages.iter().map(|x| x.source(&base)).collect::<Vec<_>>();

        for source in &sources {
            println!("Fetching: {}", source.package().name());
            source.fetch()?;
        }

        for source in &sources {
            println!("Installing: {}", source.package().name());
            source.install(self.rime_dir()?)?;
        }

        Ok(())
    }

    fn data_dir(&self) -> Result<PathBuf> {
        let data = dirs::data_local_dir().context("user profile dir unavailable")?;
        Ok(data.join("Rimeka"))
    }

    fn repos_dir(&self) -> Result<PathBuf> {
        Ok(self.data_dir()?.join("repos"))
    }

    fn rime_dir(&self) -> Result<PathBuf> {
        if let Some(x) = &self.options.dir {
            return Ok(x.clone());
        }

        let home = dirs::home_dir().context("user profile dir unavailable")?;
        let dir = match self.options.frontend {
            Frontend::Fcitx => home.join(".config/fcitx/rime"),
            Frontend::Fcitx5 => home.join(".local/share/fcitx5/rime"),
            Frontend::Ibus => home.join(".config/ibus/rime"),
            Frontend::Squirrel => home.join("Library/Rime"),
            Frontend::Weasel => home.join("AppData/Roaming/Rime"),
        };

        // TODO: remove this join
        Ok(dir.join("rimeka"))
    }
}
