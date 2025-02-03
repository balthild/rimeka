use std::path::Path;

use glob::Pattern;

use crate::Result;

pub struct PatternSet {
    patterns: Vec<Pattern>,
}

impl PatternSet {
    pub fn new<P: AsRef<str>>(patterns: &[P]) -> Result<Self> {
        let patterns = patterns
            .iter()
            .map(|p| Pattern::new(p.as_ref()))
            .try_collect()?;

        Ok(Self { patterns })
    }

    pub fn matches(&self, path: &Path) -> bool {
        for pattern in &self.patterns {
            if pattern.matches_path(path) {
                return true;
            }
        }

        false
    }
}
