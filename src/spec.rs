use std::collections::HashMap;
use std::convert::Infallible;
use std::path::Path;
use std::str::FromStr;

use anyhow::bail;

use crate::package::Package;
use crate::{lazy_regex, re_capture, re_optional, re_seplist, Result};

#[derive(Debug, Clone)]
pub struct Spec {
    repo: String,
    branch: Option<String>,
    recipe: Option<Recipe>,
    options: HashMap<String, String>,
}

impl Spec {
    pub fn resolve(target: &str) -> Result<Vec<Spec>> {
        match target {
            ":preset" => super::builtins::preset(),
            ":extra" => super::builtins::extra(),
            ":all" => super::builtins::all(),
            _ => target.parse().map(|x| vec![x]),
        }
    }

    pub fn repo(&self) -> &str {
        &self.repo
    }

    pub fn branch(&self) -> Option<&String> {
        self.branch.as_ref()
    }

    pub fn recipe(&self) -> Option<&Recipe> {
        self.recipe.as_ref()
    }

    pub fn args(&self) -> &HashMap<String, String> {
        &self.options
    }

    pub fn name(&self) -> String {
        match &self.recipe {
            Some(Recipe::Explicit(name)) => format!("{}:{}", self.repo, name),
            _ => self.repo.to_string(),
        }
    }

    pub fn locate_package(&self, base: &Path) -> Package {
        Package::new(self, base)
    }
}

impl FromStr for Spec {
    type Err = anyhow::Error;

    fn from_str(target: &str) -> Result<Self, Self::Err> {
        if target.ends_with("-packages.conf") || target.ends_with("-packages.bat") {
            bail!("*-packages.conf and *-packages.bat are not supported")
        }

        let pattern = lazy_regex! {
            "^"
            re_capture! { [repo]
                re_optional! { r"[0-9A-Za-z\-]+\/" }
                r"[0-9A-Za-z_\-\.]+"
            }
            re_optional! {
                r"\@"
                re_capture! { [branch] r"[0-9A-Za-z_\-\.\/]+" }
            }
            re_optional! {
                r"\:"
                re_capture! { [rx_name] r"[0-9A-Za-z_\-\.\/]+" }
                re_optional! {
                    r"\:"
                    re_capture! { [rx_opts]
                        re_seplist! { [","]
                            "[0-9A-Za-z]+"
                            "="
                            "[0-9A-Za-z]+"
                        }
                    }
                }
            }
            "$"
        };

        let Some(captures) = pattern.captures(target) else {
            bail!("invalid package or recipe {target}");
        };

        let mut repo = captures["repo"].to_string();
        let branch = captures.name("branch").map(|x| x.as_str());
        let rx_name = captures.name("rx_name").map(|x| x.as_str());
        let rx_opts = captures.name("rx_opts").map(|x| x.as_str()).unwrap_or("");

        if let Some(branch) = &branch {
            if branch.starts_with('.') || branch.ends_with('/') {
                bail!("invalid package or recipe {target}");
            }
        }

        if !repo.contains('/') {
            repo = format!("rime/rime-{repo}");
        }

        Ok(Self {
            repo,
            branch: branch.map(|x| x.to_string()),
            recipe: rx_name.map(|x| x.parse().unwrap()),
            options: rx_opts
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.split_once('=').unwrap())
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        })
    }
}

#[derive(Debug, Clone)]
pub enum Recipe {
    Implicit,
    Explicit(String),
}

impl Recipe {
    pub fn filename(&self) -> String {
        match self {
            Recipe::Implicit => "recipe.yaml".to_string(),
            Recipe::Explicit(name) => format!("{name}.recipe.yaml"),
        }
    }
}

impl FromStr for Recipe {
    type Err = Infallible;

    fn from_str(recipe: &str) -> Result<Self, Self::Err> {
        match recipe {
            "recipe.yaml" => Ok(Recipe::Implicit),
            name => Ok(Recipe::Explicit(
                name.strip_suffix(".recipe.yaml")
                    .unwrap_or(name)
                    .to_string(),
            )),
        }
    }
}
