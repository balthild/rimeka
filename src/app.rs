use std::path::PathBuf;

use anyhow::{bail, Context};
use owo_colors::OwoColorize;

use crate::options::{Frontend, Options};
use crate::spec::Spec;
use crate::Result;

pub struct App {
    options: Options,
}

impl App {
    pub fn new(options: Options) -> Self {
        Self { options }
    }

    pub fn run(self) -> Result {
        self.mkdir()?;

        let mut specs = self.resolve_targets()?;

        if self.options.select {
            specs = self.select(specs);
        }

        self.install(specs)
    }

    fn mkdir(&self) -> Result {
        std::fs::create_dir_all(self.data_dir()?)?;
        std::fs::create_dir_all(self.rime_dir()?)?;
        Ok(())
    }

    fn resolve_targets(&self) -> Result<Vec<Spec>> {
        let resolved = self
            .options
            .targets
            .iter()
            .map(|x| Spec::resolve(x))
            .try_collect::<Vec<_>>()?;

        Ok(resolved.concat())
    }

    fn select(&self, candidates: Vec<Spec>) -> Vec<Spec> {
        candidates
    }

    fn install(&self, specs: Vec<Spec>) -> Result {
        let base = self.packages_dir()?;

        for spec in &specs {
            println!("{} {}", "Fetching:".green(), spec.repo());
            spec.locate_package(&base).fetch()?;
        }

        for spec in &specs {
            println!("{} {}", "Installing:".green(), spec.name());
            spec.locate_package(&base).install(self.rime_dir()?)?;
        }

        Ok(())
    }

    fn data_dir(&self) -> Result<PathBuf> {
        let data = dirs::data_local_dir().context("user profile dir unavailable")?;

        #[allow(unreachable_patterns)]
        let name = match true {
            cfg!(target_os = "windows") => "Rimeka",
            cfg!(target_os = "macos") => "Rimeka",
            cfg!(unix) => "rimeka",
            _ => bail!("unsupported operating system"),
        };

        Ok(data.join(name))
    }

    fn packages_dir(&self) -> Result<PathBuf> {
        Ok(self.data_dir()?.join("packages"))
    }

    fn rime_dir(&self) -> Result<PathBuf> {
        if let Some(x) = &self.options.dir {
            return Ok(x.clone());
        }

        let home = dirs::home_dir().context("user profile dir unavailable")?;
        let dir = match self.options.frontend.or_guess() {
            Frontend::Fcitx => home.join(".config/fcitx/rime"),
            Frontend::Fcitx5 => home.join(".local/share/fcitx5/rime"),
            Frontend::Ibus => home.join(".config/ibus/rime"),
            Frontend::Squirrel => home.join("Library/Rime"),
            Frontend::Weasel => home.join("AppData/Roaming/Rime"),
            Frontend::Unknown => {
                bail!("Cannot determine frontend. Please specify --frontend or --dir")
            }
        };

        // TODO: remove this join
        Ok(dir.join("rimeka"))
    }
}
