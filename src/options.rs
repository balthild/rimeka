use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::bail;
use bpaf::{Args, Bpaf, ParseFailure};

#[derive(Debug, Bpaf)]
#[bpaf(options, version, fallback_to_usage)]
pub struct Options {
    /// Select package interactively
    #[bpaf(short, long)]
    pub select: bool,

    /// Specify the RIME frontend
    #[bpaf(short, long, fallback(Frontend::default()))]
    pub frontend: Frontend,

    /// Specify the directory of RIME configurations
    #[bpaf(short, long)]
    pub dir: Option<PathBuf>,

    /// Specify packages or recipes to be installed
    #[bpaf(positional("targets"))]
    pub targets: Vec<String>,
}

impl Options {
    pub fn parse() -> Self {
        let parser = options();

        match parser.run_inner(Args::current_args()) {
            Ok(mut options) => {
                if options.targets.is_empty() {
                    options.targets.push(":preset".to_string());
                }

                options
            }
            Err(err) => {
                err.print_message(80);

                if let ParseFailure::Stderr(..) = err {
                    println!();
                    parser.run_inner(&["--help"]).unwrap_err().print_message(80)
                }

                std::process::exit(err.exit_code())
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Frontend {
    Fcitx,
    Fcitx5,
    Ibus,
    Squirrel,
    Weasel,
    Unknown,
}

impl Display for Frontend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Frontend::Fcitx => "fcitx/fcitx-rime",
            Frontend::Fcitx5 => "fcitx/fcitx5-rime",
            Frontend::Ibus => "rime/ibus-rime",
            Frontend::Squirrel => "rime/squirrel",
            Frontend::Weasel => "rime/weasel",
            Frontend::Unknown => "unknown",
        };

        f.write_str(name)
    }
}

impl Default for Frontend {
    fn default() -> Self {
        cfg_match! {
            target_os = "linux" => { Self::Ibus }
            target_os = "macos" => { Self::Squirrel }
            target_os = "windows" => { Self::Weasel }
            _ => { Self::Unknown }
        }
    }
}

impl FromStr for Frontend {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fcitx/fcitx-rime" | "fcitx-rime" => Ok(Self::Fcitx),
            "fcitx/fcitx5-rime" | "fcitx5/fcitx5-rime" | "fcitx5-rime" => Ok(Self::Fcitx5),
            "rime/ibus-rime" | "ibus-rime" => Ok(Self::Ibus),
            "rime/squirrel" | "squirrel" => Ok(Self::Squirrel),
            "rime/weasel" | "weasel" => Ok(Self::Weasel),
            _ => bail!("unknown frontend: {s}"),
        }
    }
}
