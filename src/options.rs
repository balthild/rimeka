use std::path::PathBuf;
use std::str::FromStr;

use anyhow::bail;
use bpaf::{Args, Bpaf, ParseFailure};

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version, fallback_to_usage)]
pub struct Options {
    /// Select package interactively
    #[bpaf(short, long)]
    pub select: bool,

    /// Specify the RIME frontend
    #[bpaf(short, long, fallback(Default::default()))]
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

#[derive(Debug, Clone)]
pub enum Frontend {
    Fcitx,
    Fcitx5,
    Ibus,
    Squirrel,
    Weasel,
}

impl Default for Frontend {
    #[cfg(target_os = "linux")]
    fn default() -> Self {
        Self::Ibus
    }

    #[cfg(target_os = "macos")]
    fn default() -> Self {
        Self::Squirrel
    }

    #[cfg(target_os = "windows")]
    fn default() -> Self {
        Self::Weasel
    }
}

impl FromStr for Frontend {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fcitx/fcitx-rime" | "fcitx-rime" => Ok(Self::Fcitx),
            "fcitx5/fcitx5-rime" | "fcitx5-rime" => Ok(Self::Fcitx5),
            "rime/ibus-rime" | "ibus-rime" => Ok(Self::Ibus),
            "rime/squirrel" | "squirrel" => Ok(Self::Squirrel),
            "rime/weasel" | "weasel" => Ok(Self::Weasel),
            _ => bail!("unknown frontend: {s}"),
        }
    }
}
