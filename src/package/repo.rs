use std::path::{Path, PathBuf};

use git2::build::RepoBuilder;
use git2::{Direction, FetchOptions, Repository, ResetType};

use crate::Result;

use super::Package;

pub struct GitHubRepo {
    pub url: String,
    pub dir: PathBuf,
    pub branch: Option<String>,
}

impl GitHubRepo {
    pub fn new(package: &Package, dir: &Path) -> Self {
        GitHubRepo {
            url: format!("https://github.com/{}", package.name),
            dir: dir.to_path_buf(),
            branch: package.branch.clone(),
        }
    }

    pub fn pull_or_clone(&self) -> Result {
        if self.dir.join(".git").is_dir() {
            self.pull()
        } else {
            self.clone()
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
