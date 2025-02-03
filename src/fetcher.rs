use std::path::{Path, PathBuf};

use crate::spec::Spec;
use crate::Result;

pub struct GitFetcher {
    url: String,
    dir: PathBuf,
    branch: Option<String>,
}

impl GitFetcher {
    pub fn new(spec: &Spec, dir: &Path) -> Self {
        Self {
            url: format!("https://github.com/{}", spec.repo()),
            dir: dir.to_path_buf(),
            branch: spec.branch().map(|x| x.to_string()),
        }
    }
}

#[cfg(feature = "git-cli")]
impl GitFetcher {
    pub fn clone(&self) -> Result {
        std::fs::create_dir_all(&self.dir)?;
        self.call("git", &["clone", &self.url, ".", "--depth=1"])?;

        Ok(())
    }

    pub fn pull(&self) -> Result {
        let head = self.branch.as_deref().unwrap_or("HEAD");
        self.call("git", &["remote", "set-head", "--auto", "origin"])?;
        self.call("git", &["fetch", "origin", head, "--depth=1"])?;
        self.call("git", &["reset", "--hard", &format!("origin/{}", head)])?;
        self.call("git", &["clean", "-xdf"])?;

        Ok(())
    }

    fn call(&self, command: &str, args: &[&str]) -> Result {
        use std::process::Command;

        Command::new(command)
            .current_dir(&self.dir)
            .args(args)
            .spawn()?
            .wait()?;

        Ok(())
    }
}

#[cfg(feature = "git-libgit2")]
impl GitFetcher {
    pub fn clone(&self) -> Result {
        use git2::build::RepoBuilder;
        use git2::FetchOptions;

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

    pub fn pull(&self) -> Result {
        use git2::{Direction, FetchOptions, Repository, ResetType};

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
