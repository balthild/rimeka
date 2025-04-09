use std::path::{Path, PathBuf};

use crate::spec::Spec;

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
mod cli {
    use std::io::BufRead;
    use std::process::Command;

    use anyhow::Context;

    use crate::Result;

    impl super::GitFetcher {
        pub fn clone(&self) -> Result {
            std::fs::create_dir_all(&self.dir)?;

            let mut command = Command::new("git");
            command.current_dir(&self.dir);
            command.arg("clone");
            command.arg(&self.url);
            command.arg(&self.dir);
            command.arg("--depth=1");
            if let Some(branch) = &self.branch {
                command.args(["--branch", branch]);
            }

            command.spawn()?.wait()?.exit_ok()?;

            println!();
            Ok(())
        }

        pub fn pull(&self) -> Result {
            let branch = match &self.branch {
                Some(branch) => branch.clone(),
                None => self.get_default_branch()?,
            };
            let upstream = format!("origin/{branch}");

            self.call("git", &["clean", "-xdf"])?;
            self.call("git", &["reset", "--hard", "HEAD"])?;
            self.call("git", &["fetch", "origin", &branch, "--depth=1"])?;
            self.call("git", &["switch", "-C", &branch, "--track", &upstream])?;

            println!();
            Ok(())
        }

        fn get_default_branch(&self) -> Result<String> {
            let output = Command::new("git")
                .current_dir(&self.dir)
                .args(["ls-remote", "--symref", "origin", "HEAD"])
                .output()?;

            output.status.exit_ok()?;

            output
                .stdout
                .lines()
                .map_while(|line| line.ok())
                .find_map(|line| {
                    line.strip_prefix("ref: refs/heads/")
                        .and_then(|x| x.strip_suffix("\tHEAD"))
                        .map(str::trim)
                        .map(String::from)
                })
                .context("unexpected output from `git ls-remote`")
        }

        fn call(&self, command: &str, args: &[&str]) -> Result {
            Command::new(command)
                .current_dir(&self.dir)
                .args(args)
                .spawn()?
                .wait()?
                .exit_ok()?;

            Ok(())
        }
    }
}

#[cfg(feature = "git-libgit2")]
mod libgit2 {
    use git2::build::RepoBuilder;
    use git2::{Direction, FetchOptions, Repository, ResetType};

    use crate::Result;

    impl super::GitFetcher {
        pub fn clone(&self) -> Result {
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
}
