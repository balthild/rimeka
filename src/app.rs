use std::ffi::OsStr;
use std::path::PathBuf;

use anyhow::{bail, Context};
use chumsky::container::Seq;
use dialoguer::theme::SimpleTheme;
use dialoguer::MultiSelect;
use owo_colors::OwoColorize;
use path_clean::PathClean;
use pathdiff::diff_paths;
use walkdir::WalkDir;

use crate::options::{Frontend, Options};
use crate::spec::Spec;
use crate::Result;

pub struct App {
    options: Options,
    rime_dir: PathBuf,
    data_dir: PathBuf,
    packages_dir: PathBuf,
}

impl App {
    pub fn new(options: Options) -> Self {
        Self {
            options,
            rime_dir: PathBuf::new(),
            data_dir: PathBuf::new(),
            packages_dir: PathBuf::new(),
        }
    }

    pub fn run(mut self) -> Result {
        self.initialize()?;

        if self.options.list {
            return self.list();
        }

        self.banner();

        let mut specs = self.resolve()?;

        if self.options.select {
            specs = self.select(specs);
            if specs.is_empty() {
                bail!("no package is selected")
            }
        }

        self.install(specs)
    }

    fn initialize(&mut self) -> Result {
        self.rime_dir = Self::find_rime_dir(&self.options)?;
        self.data_dir = Self::find_data_dir()?;
        self.packages_dir = self.data_dir.join("packages");

        std::fs::create_dir_all(&self.rime_dir)?;
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(&self.packages_dir)?;

        Ok(())
    }

    fn list(&self) -> Result {
        for entry in WalkDir::new(&self.packages_dir)
            .min_depth(2)
            .max_depth(2)
            .into_iter()
            .filter_map(|x| x.ok())
            .filter(|x| x.file_type().is_dir())
        {
            let repo_path = entry.path();
            let repo = diff_paths(repo_path, &self.packages_dir)
                .context("walked path shouldn't be relative")?
                .to_string_lossy()
                .into_owned();

            let repo = repo.strip_prefix("rime/rime-").unwrap_or(&repo);
            println!("{repo}");

            for entry in WalkDir::new(repo_path)
                .into_iter()
                .filter_entry(|x| x.file_name() != OsStr::new(".git"))
                .filter_map(|x| x.ok())
                .filter(|x| x.file_type().is_file())
            {
                let recipe_path = entry.path();
                let recipe = diff_paths(recipe_path, repo_path)
                    .context("walked path shouldn't be relative")?
                    .to_string_lossy()
                    .into_owned();

                if let Some(recipe) = recipe.strip_suffix(".recipe.yaml") {
                    println!("{repo}:{recipe}");
                }
            }
        }

        Ok(())
    }

    fn banner(&self) {
        let frontend = match self.options.dir {
            Some(_) => Frontend::Unknown,
            None => self.options.frontend,
        };

        println!("Installing for RIME frontend: {}", frontend.magenta());
        println!();

        println!("RIME User Directory: {}", self.rime_dir.display());
        println!("Packages Directory: {}", self.packages_dir.display());
        println!();
    }

    fn resolve(&self) -> Result<Vec<Spec>> {
        let resolved = self
            .options
            .targets
            .iter()
            .map(|x| Spec::resolve(x))
            .try_collect::<Vec<_>>()?;

        Ok(resolved.concat())
    }

    fn select(&self, candidates: Vec<Spec>) -> Vec<Spec> {
        let choices = MultiSelect::with_theme(&SimpleTheme)
            .with_prompt("Pick the packages to be installed, or press Ctrl+C to cancel")
            .items(&candidates)
            .interact()
            .unwrap_or_else(|_| std::process::exit(1));

        candidates
            .into_iter()
            .enumerate()
            .filter(|(i, _)| choices.contains(i))
            .map(|(_, spec)| spec)
            .collect()
    }

    fn install(&self, specs: Vec<Spec>) -> Result {
        for spec in &specs {
            println!("{} {}", "Fetching:".green(), spec.repo(),);
            spec.locate_package(&self.packages_dir).fetch()?;
        }

        for spec in &specs {
            println!("{} {}", "Installing:".green(), spec.name());
            spec.locate_package(&self.packages_dir)
                .install(self.rime_dir.clone())?;
        }

        Ok(())
    }

    fn find_rime_dir(options: &Options) -> Result<PathBuf> {
        if let Some(x) = &options.dir {
            return Ok(x.clean());
        }

        let home = dirs::home_dir().context("user profile dir unavailable")?;

        let dir = match options.frontend {
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

    fn find_data_dir() -> Result<PathBuf> {
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
}
