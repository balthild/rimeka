use std::path::{Path, PathBuf};

use anyhow::Context;
use owo_colors::OwoColorize;
use path_clean::PathClean;

use crate::fetcher::GitFetcher;
use crate::installer::{DefaultInstaller, RecipeInstaller};
use crate::spec::{Recipe, Spec};
use crate::Result;

#[derive(Debug)]
pub struct Package<'a> {
    spec: &'a Spec,
    dir: PathBuf,
}

impl<'a> Package<'a> {
    pub fn new(spec: &'a Spec, base: &Path) -> Self {
        let dir = base.join(spec.repo()).clean();
        Self { spec, dir }
    }

    pub fn spec(&self) -> &Spec {
        self.spec
    }

    pub fn dir(&self) -> &PathBuf {
        &self.dir
    }

    pub fn fetch(&self) -> Result {
        let fetcher = GitFetcher::new(self.spec, &self.dir);
        if self.dir.join(".git").is_dir() {
            fetcher.pull()
        } else {
            fetcher.clone()
        }
    }

    pub fn install(&self, dest: PathBuf) -> Result {
        for (k, v) in self.spec.options() {
            println!("- {} {} = {}", "Option:".cyan(), k, v);
        }

        if let Some(recipe) = self.spec.recipe() {
            let installer = RecipeInstaller::new(self, dest, recipe.clone());
            return installer.install().context("failed to install recipe");
        }

        if self.dir.join(Recipe::Implicit.filename()).exists() {
            let installer = RecipeInstaller::new(self, dest, Recipe::Implicit);
            return installer.install().context("failed to install recipe");
        }

        DefaultInstaller::new(self, dest).install()
    }
}
