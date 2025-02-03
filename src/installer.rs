use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use owo_colors::OwoColorize;
use path_clean::PathClean;
use pathdiff::diff_paths;
use saphyr::{Hash, Yaml, YamlEmitter};
use walkdir::WalkDir;

use crate::glob::PatternSet;
use crate::package::Package;
use crate::spec::Recipe;
use crate::Result;

#[derive(Debug)]
pub struct RecipeInstaller<'a> {
    package: &'a Package<'a>,
    dest: PathBuf,
    recipe: Recipe,
    options: HashMap<String, String>,
}

impl<'a> RecipeInstaller<'a> {
    pub fn new(package: &'a Package, dest: PathBuf, recipe: Recipe) -> Self {
        Self {
            package,
            dest,
            recipe,
            options: HashMap::new(),
        }
    }

    pub fn install(mut self) -> Result {
        let path = self.package.dir().join(self.recipe.filename()).clean();
        let yaml = std::fs::read_to_string(&path).context("failed to read file")?;
        let docs = Yaml::load_from_str(&yaml).context("failed to parse yaml")?;

        let doc = &docs[0];

        self.resolve_options(&doc["recipe"]);

        if let Some(patterns) = doc["install_files"].as_str() {
            self.install_files(patterns)
                .context("failed to install files")?;
        }

        if let Some(patches) = doc["patch_files"].as_hash() {
            self.install_patches(patches)
                .context("failed to install patches")?;
        }

        Ok(())
    }

    fn resolve_options(&mut self, meta: &Yaml) {
        // Default options defined in the YAML
        if let Some(args) = meta["args"].as_vec() {
            self.options.extend(
                args.iter()
                    .filter_map(|x| x.as_str())
                    .filter_map(|x| x.split_once('='))
                    .map(|(k, v)| (k.to_string(), v.to_string())),
            );
        }

        // Overriden options specified in the CLI args
        self.options.extend(self.package.spec().args().clone());
    }

    fn install_files(&self, patterns: &str) -> Result {
        install_dir(
            self.package.dir(),
            &self.dest,
            &shlex::split(patterns).context("syntax error in the file list")?,
            &[],
        )
    }

    fn install_patches(&self, patches: &Hash) -> Result {
        for (filename, patch) in patches {
            let filename = filename.as_str().context("filename must be a string")?;
            patch.as_vec().context("patch must be an array")?;

            println!("- {} {}", "Patching:".cyan(), filename);
            self.install_patch(filename, patch)?;
        }

        Ok(())
    }

    fn install_patch(&self, filename: &str, patch: &Yaml) -> Result {
        let path = self.dest.join(filename).clean();

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

        let name = self.package.spec().name();
        let header = format!("# Rx: {name}: {{");
        let footer = "# }".to_string();

        if let Some(line_top) = yaml.iter().position(|x| x == &header) {
            let Some(line_delta) = yaml.iter().skip(line_top).position(|x| x == &footer) else {
                bail!("failed to parse the previously patched file");
            };

            let line_bottom = line_top + line_delta;
            yaml.drain(line_top..=line_bottom);
        }

        let mut out = String::new();
        let mut emitter = YamlEmitter::new(&mut out);
        emitter.dump(patch)?;

        yaml.push(header);
        yaml.extend(out.lines().skip(1).map(|line| {
            let line = format!("  {line}");
            let line = shellexpand::env_with_context_no_errors(&line, |key| self.options.get(key));
            line.to_string()
        }));
        yaml.push(footer);

        std::fs::write(path, yaml.join("\n") + "\n")?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct DefaultInstaller<'a> {
    package: &'a Package<'a>,
    dest: PathBuf,
}

impl<'a> DefaultInstaller<'a> {
    pub fn new(package: &'a Package, dest: PathBuf) -> Self {
        Self { package, dest }
    }

    pub fn install(self) -> Result {
        install_dir(
            self.package.dir(),
            &self.dest,
            &["*.yaml", "*.txt", "*.gram", "opencc/*.*"],
            &[
                "recipe.yaml",
                "**/*.recipe.yaml",
                "**/*.custom.yaml",
                "**/*.json",
                "**/*.ocd",
                "**/*.txt",
            ],
        )
    }
}

fn install_dir<P>(src: &Path, dest: &Path, include: &[P], exclude: &[P]) -> Result
where
    P: AsRef<str>,
{
    let include = PatternSet::new(include)?;
    let exclude = PatternSet::new(exclude)?;

    for entry in WalkDir::new(src) {
        let entry = entry?;
        let relative = diff_paths(entry.path(), src).expect("walked path shouldn't be relative");

        if !include.matches(&relative) || exclude.matches(&relative) {
            continue;
        }

        println!("- {} {}", "Copying:".cyan(), relative.display());

        let to = dest.join(&relative);
        std::fs::create_dir_all(to.parent().unwrap())?;
        std::fs::copy(entry.path(), to)?;
    }

    Ok(())
}
