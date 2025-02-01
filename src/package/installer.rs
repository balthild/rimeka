use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use globset::{Glob, GlobSet, GlobSetBuilder};
use pathdiff::diff_paths;
use saphyr::{Hash, Yaml, YamlEmitter};
use walkdir::WalkDir;

use crate::Result;

use super::source::PackageSource;
use super::Recipe;

#[derive(Debug, Clone)]
pub struct RecipeInstaller<'a> {
    source: &'a PackageSource<'a>,
    dest: PathBuf,
    recipe: Recipe,
}

impl<'a> RecipeInstaller<'a> {
    pub fn new(source: &'a PackageSource, dest: PathBuf, recipe: Recipe) -> Self {
        Self {
            source,
            dest,
            recipe,
        }
    }

    pub fn install(&self) -> Result {
        let path = self.source.dir().join(self.recipe.filename());

        let yaml = std::fs::read_to_string(&path).context("failed to read file")?;
        let docs = Yaml::load_from_str(&yaml).context("failed to parse yaml")?;

        if let Some(patterns) = docs[0]["install_files"].as_str() {
            self.install_files(&docs[0], patterns)
                .context("failed to install files")?;
        }

        if let Some(patches) = docs[0]["patch_files"].as_hash() {
            self.install_patches(&docs[0], patches)
                .context("failed to install patches")?;
        }

        Ok(())
    }

    fn install_files(&self, doc: &Yaml, patterns: &str) -> Result {
        install_dir(
            self.source.dir(),
            &self.dest,
            &shlex::split(patterns).context("syntax error in the file list")?,
            &[],
        )
    }

    fn install_patches(&self, doc: &Yaml, patches: &Hash) -> Result {
        for (filename, patch) in patches {
            let filename = filename.as_str().context("filename must be a string")?;
            patch.as_vec().context("patch must be an array")?;

            println!("Patching: {filename}");
            self.install_patch(doc, filename, patch)?;
        }

        Ok(())
    }

    fn install_patch(&self, doc: &Yaml, filename: &str, patch: &Yaml) -> Result {
        let path = self.dest.join(filename);

        let yaml = if path.exists() {
            std::fs::read_to_string(&path)?
        } else {
            String::new()
        };

        // Processing the YAML as plain text rather than calling a YAML library
        // because /plum/ uses comments to keep track of the patches it installs
        let mut yaml = yaml.lines().map(|x| x.to_string()).collect::<Vec<_>>();

        if !yaml.iter().any(|x| x == "__patch:") {
            yaml.push("__patch:".to_string());
        }

        let rxid = self.recipe.rxid(self.source.package());
        let header = format!("# Rx: {rxid}: {{");
        let footer = "# }".to_string();

        if let Some(line_top) = yaml.iter().position(|x| x == &header) {
            let Some(line_delta) = yaml.iter().skip(line_top).position(|x| x == &footer) else {
                bail!("failed to parse previous patch file");
            };

            let line_bottom = line_top + line_delta;
            yaml.drain(line_top..=line_bottom);
        }

        let mut out = String::new();
        let mut emitter = YamlEmitter::new(&mut out);
        emitter.dump(patch)?;

        let default_args = self.get_default_args(doc);

        yaml.push(header);
        yaml.extend(out.lines().skip(1).map(|line| {
            shellexpand::env_with_context_no_errors(&format!("  {line}"), |key| {
                match self.recipe.args.get(key) {
                    Some(value) => Some(&**value),
                    None => default_args.get(key).map(|v| &**v),
                }
            })
            .to_string()
        }));
        yaml.push(footer);

        std::fs::write(path, yaml.join("\n") + "\n")?;

        Ok(())
    }

    fn get_default_args<'b>(&self, doc: &'b Yaml) -> HashMap<&'b str, &'b str> {
        doc["recipe"]["args"]
            .as_vec()
            .map(|args| {
                args.iter()
                    .filter_map(|x| x.as_str())
                    .filter_map(|x| x.split_once('='))
                    .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct DefaultInstaller<'a> {
    source: &'a PackageSource<'a>,
    dest: PathBuf,
}

impl<'a> DefaultInstaller<'a> {
    pub fn new(source: &'a PackageSource, dest: PathBuf) -> Self {
        Self { source, dest }
    }

    pub fn install(&self) -> Result {
        install_dir(
            self.source.dir(),
            &self.dest,
            &["*.yaml", "*.txt", "*.gram", "opencc/*.*"],
            &[
                "**/recipe.yaml",
                "**/*.{recipe.yaml,custom.yaml,json,ocd,txt}",
            ],
        )
    }
}

fn install_dir<P>(src: &Path, dest: &Path, include: &[P], exclude: &[P]) -> Result
where
    P: AsRef<str>,
{
    let include = build_glob_set(include)?;
    let exclude = build_glob_set(exclude)?;

    for entry in WalkDir::new(src) {
        let entry = entry?;
        let relative = diff_paths(entry.path(), src).expect("path shouldn't be relative");

        if !include.is_match(&relative) || exclude.is_match(&relative) {
            continue;
        }

        println!("Copying: {}", relative.display());

        let to = dest.join(&relative);
        std::fs::create_dir_all(to.parent().unwrap())?;
        std::fs::copy(entry.path(), to)?;
    }

    Ok(())
}

fn build_glob_set<P>(patterns: &[P]) -> Result<GlobSet>
where
    P: AsRef<str>,
{
    let mut builder = GlobSetBuilder::new();

    for pattern in patterns {
        let pattern = pattern.as_ref();
        let glob = Glob::new(pattern);
        builder.add(glob.with_context(|| format!("invalid glob pattern: {pattern}"))?);
    }

    builder.build().context("failed to build glob set")
}
