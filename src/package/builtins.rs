use std::str::FromStr;

use crate::Result;

use super::Package;

pub fn preset() -> Result<Vec<Package>> {
    [
        "bopomofo",
        "cangjie",
        "essay",
        "luna-pinyin",
        "prelude",
        "stroke",
        "terra-pinyin",
    ]
    .into_iter()
    .map(Package::from_str)
    .collect()
}

pub fn extra() -> Result<Vec<Package>> {
    [
        "array",
        "cantonese",
        "combo-pinyin",
        "double-pinyin",
        "emoji",
        "ipa",
        "jyutping",
        "middle-chinese",
        "pinyin-simp",
        "quick",
        "scj",
        "soutzoe",
        "stenotype",
        "wubi",
        "wugniu",
    ]
    .into_iter()
    .map(Package::from_str)
    .collect()
}

pub fn all() -> Result<Vec<Package>> {
    let preset = preset()?;
    let extra = extra()?;
    Ok(preset.into_iter().chain(extra).collect())
}
