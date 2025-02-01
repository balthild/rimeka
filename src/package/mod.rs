use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use anyhow::bail;
use source::PackageSource;

use crate::{lazy_regex, re_capture, re_optional, re_seplist, Result};

mod builtins;
mod installer;
mod repo;
mod source;

#[derive(Debug, Clone)]
pub struct Recipe {
    name: String,
    args: HashMap<String, String>,
}

impl Recipe {
    fn implicit() -> Self {
        Self {
            name: "recipe.yaml".to_string(),
            args: HashMap::new(),
        }
    }

    fn filename(&self) -> String {
        if self.name == "recipe.yaml" {
            return self.name.clone();
        }

        format!("{}.recipe.yaml", self.name)
    }

    fn rxid(&self, package: &Package) -> String {
        let mut rxid = package.name.to_string();

        rxid.push(':');
        rxid.push_str(&self.name);

        if !self.args.is_empty() {
            let options = self
                .args
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(",");

            rxid.push(':');
            rxid.push_str(&options);
        }

        rxid
    }
}

#[derive(Debug, Clone)]
pub struct Package {
    name: String,
    branch: Option<String>,
    recipe: Option<Recipe>,
}

impl Package {
    pub fn resolve(target: &str) -> Result<Vec<Package>> {
        match target {
            ":preset" => builtins::preset(),
            ":extra" => builtins::extra(),
            ":all" => builtins::all(),
            _ => target.parse().map(|x| vec![x]),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source(&self, base: &Path) -> PackageSource {
        PackageSource::new(self, base)
    }
}

impl FromStr for Package {
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
                    re_capture! { [rx_args]
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
        let rx_args = captures.name("rx_args").map(|x| x.as_str()).unwrap_or("");

        if let Some(branch) = &branch {
            if branch.starts_with('.') || branch.ends_with('/') {
                bail!("invalid package or recipe {target}");
            }
        }

        if !repo.contains('/') {
            repo = format!("rime/rime-{repo}");
        }

        Ok(Self {
            name: repo,
            branch: branch.map(|x| x.to_string()),
            recipe: rx_name.map(|x| Recipe {
                name: x.strip_suffix(".recipe.yaml").unwrap_or(x).to_string(),
                args: rx_args
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.split_once('=').unwrap())
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            }),
        })
    }
}
