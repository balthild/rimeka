use std::path::PathBuf;

use anyhow::{bail, Context};
use owo_colors::OwoColorize;
use path_clean::PathClean;

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
        let rime = self.rime_dir()?;
        let data = self.data_dir()?;
        let packages = self.packages_dir()?;

        println!("RIME User Directory: {}", rime.display());
        println!("Rimeka Directory: {}", data.display());
        println!("Packages Directory: {}", packages.display());
        println!();

        std::fs::create_dir_all(rime)?;
        std::fs::create_dir_all(data)?;
        std::fs::create_dir_all(packages)?;

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
            println!("{} {}", "Fetching:".green(), spec.repo(),);
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
            return Ok(x.clean());
        }

        let home = dirs::home_dir().context("user profile dir unavailable")?;

        let dir = match self.options.frontend.or_guess() {
            Frontend::Fcitx => ".config/fcitx/rime",
            Frontend::Fcitx5 => ".local/share/fcitx5/rime",
            Frontend::Ibus => ".config/ibus/rime",
            Frontend::Squirrel => "Library/Rime",
            Frontend::Weasel => "AppData/Roaming/Rime",
            Frontend::Unknown => {
                bail!("--frontend or --dir is required on this operating system")
            }
        };

        Ok(home.join(dir).clean())
    }
}
