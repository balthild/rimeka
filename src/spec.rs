use std::collections::HashMap;
use std::convert::Infallible;
use std::path::Path;
use std::str::FromStr;

use anyhow::{anyhow, bail};

use crate::package::Package;
use crate::Result;

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

    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
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
        use chumsky::prelude::*;

        type Extra<'s> = extra::Err<Rich<'s, char>>;

        fn alnum<'s>() -> impl Parser<'s, &'s str, char, Extra<'s>> {
            any().filter(|c: &char| c.is_alphanumeric())
        }

        fn username<'s>() -> impl Parser<'s, &'s str, String, Extra<'s>> {
            alnum().or(just('-')).repeated().at_least(1).collect()
        }

        fn reponame<'s>() -> impl Parser<'s, &'s str, String, Extra<'s>> {
            alnum().or(one_of("._-")).repeated().at_least(1).collect()
        }

        fn pathname<'s>(dotfile: bool) -> impl Parser<'s, &'s str, String, Extra<'s>> {
            let token = alnum().or(one_of("._-"));
            let item = token.repeated().at_least(1).collect();
            let item = item.filter(move |x: &String| dotfile || !x.starts_with('.'));
            let seq = item.separated_by(just('/')).at_least(1).collect::<Vec<_>>();
            seq.map(|x| x.join("."))
        }

        pub fn repo<'s>() -> impl Parser<'s, &'s str, String, Extra<'s>> {
            let community = username().then_ignore(just('/')).then(reponame());
            let builtins = reponame().map(|x| format!("rime/rime-{x}"));
            community.map(|(a, b)| format!("{a}/{b}")).or(builtins)
        }

        pub fn branch<'s>() -> impl Parser<'s, &'s str, Option<String>, Extra<'s>> {
            just('@').ignore_then(pathname(false)).or_not()
        }

        pub fn recipe<'s>() -> impl Parser<'s, &'s str, Option<String>, Extra<'s>> {
            just(':').ignore_then(pathname(true)).or_not()
        }

        pub fn options<'s>() -> impl Parser<'s, &'s str, HashMap<String, String>, Extra<'s>> {
            let key = alnum().or(just('_')).repeated().at_least(1).collect();
            let value = alnum().or(just('_')).repeated().at_least(1).collect();
            let entry = key.then_ignore(just('=')).then(value);
            let list = entry.separated_by(just(',')).collect();
            let default = empty().map(|_| HashMap::new());
            just(':').ignore_then(list).or(default)
        }

        pub fn parser<'s>() -> impl Parser<
            's,
            &'s str,
            (
                String,
                Option<String>,
                Option<String>,
                HashMap<String, String>,
            ),
            Extra<'s>,
        > {
            group((repo(), branch(), recipe(), options())).then_ignore(end())
        }

        if target.ends_with("-packages.conf") || target.ends_with("-packages.bat") {
            bail!("*-packages.conf and *-packages.bat are not supported")
        }

        let (repo, branch, recipe, options) =
            parser().parse(target.trim()).into_result().map_err(|e| {
                let first = e[0].to_string();
                anyhow!("invalid package or recipe: {target}\n{first}")
            })?;

        Ok(Self {
            repo: repo.to_string(),
            branch: branch.map(|x| x.to_string()),
            recipe: recipe.map(|x| x.parse().unwrap()),
            options,
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
