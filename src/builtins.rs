use std::str::FromStr;

use crate::Result;
use crate::Spec;

pub fn preset() -> Result<Vec<Spec>> {
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
    .map(Spec::from_str)
    .collect()
}

pub fn extra() -> Result<Vec<Spec>> {
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
    .map(Spec::from_str)
    .collect()
}

pub fn all() -> Result<Vec<Spec>> {
    let preset = preset()?;
    let extra = extra()?;
    Ok(preset.into_iter().chain(extra).collect())
}
