use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context;

use crate::spec::Spec;
use crate::Result;

pub struct GitHubFetcher {
    url: String,
    dir: PathBuf,
    branch: Option<String>,
}

impl GitHubFetcher {
    pub fn new(spec: &Spec, dir: &Path) -> Self {
        Self {
            url: format!("https://github.com/{}", spec.repo()),
            dir: dir.to_path_buf(),
            branch: spec.branch().map(|x| x.to_string()),
        }
    }

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

        // The `Lines` instance comes from a `Vec<u8>`, so it will never produce I/O errors.
        #[allow(clippy::lines_filter_map_ok)]
        output
            .stdout
            .lines()
            .filter_map(|line| line.ok())
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
