use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::Result;

use super::installer::{DefaultInstaller, RecipeInstaller};
use super::repo::GitHubRepo;
use super::{Package, Recipe};

#[derive(Debug, Clone)]
pub struct PackageSource<'a> {
    package: &'a Package,
    dir: PathBuf,
}

impl<'a> PackageSource<'a> {
    pub fn new(package: &'a Package, base: &Path) -> Self {
        Self {
            package,
            dir: base.join(&package.name),
        }
    }

    pub fn package(&self) -> &Package {
        self.package
    }

    pub fn dir(&self) -> &PathBuf {
        &self.dir
    }

    pub fn fetch(&self) -> Result {
        GitHubRepo::new(self.package, &self.dir).pull_or_clone()
    }

    pub fn install(&self, dest: PathBuf) -> Result {
        if let Some(recipe) = &self.package.recipe {
            let installer = RecipeInstaller::new(self, dest, recipe.clone());
            return installer.install().context("failed to install recipe");
        }

        if self.dir.join("recipe.yaml").exists() {
            let installer = RecipeInstaller::new(self, dest, Recipe::implicit());
            return installer.install().context("failed to install recipe");
        }

        DefaultInstaller::new(self, dest).install()
    }
}
