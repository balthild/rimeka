use std::path::{Path, PathBuf};

use anyhow::Context;
use git2::build::RepoBuilder;
use git2::{Direction, FetchOptions, Repository, ResetType};
use owo_colors::OwoColorize;

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
        let dir = base.join(spec.repo());
        Self { spec, dir }
    }

    pub fn spec(&self) -> &Spec {
        self.spec
    }

    pub fn dir(&self) -> &PathBuf {
        &self.dir
    }

    pub fn fetch(&self) -> Result {
        let repo = GitHubRepo::new(self.spec, &self.dir);
        if self.dir.join(".git").is_dir() {
            repo.pull()
        } else {
            repo.clone()
        }
    }

    pub fn install(&self, dest: PathBuf) -> Result {
        for (k, v) in self.spec.args() {
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

struct GitHubRepo {
    url: String,
    dir: PathBuf,
    branch: Option<String>,
}

impl GitHubRepo {
    fn new(spec: &Spec, dir: &Path) -> Self {
        GitHubRepo {
            url: format!("https://github.com/{}", spec.repo()),
            dir: dir.to_path_buf(),
            branch: spec.branch().cloned(),
        }
    }

    fn clone(&self) -> Result {
        let mut builder = RepoBuilder::new();

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.depth(1);
        builder.fetch_options(fetch_opts);

        if let Some(branch) = &self.branch {
            builder.branch(branch);
        }

        builder.clone(&self.url, &self.dir)?;

        Ok(())
    }

    fn pull(&self) -> Result {
        let repo = Repository::open(&self.dir)?;

        let mut remote = repo.find_remote("origin")?;
        remote.connect(Direction::Fetch)?;

        let branch = match &self.branch {
            Some(branch) => branch.clone(),
            None => String::from_utf8(remote.default_branch()?.to_vec())?,
        };

        remote.fetch(&[branch], Some(FetchOptions::new().depth(1)), None)?;

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let commit = fetch_head.peel_to_commit()?.into_object();

        repo.reset(&commit, ResetType::Hard, None)?;

        Ok(())
    }
}
