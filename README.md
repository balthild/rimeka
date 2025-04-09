# Rimeka

Rime configuration installer. An alternative to [/plum/](https://github.com/rime/plum/).

## Installation

With cargo-binstall:

```shell
cargo binstall rimeka
```

Manual installation:

Download the binary from [releases](https://github.com/balthild/rimeka/releases).

## Usage

```
Usage: rimeka.exe [-s] [-f=ARG] [-d=ARG] [<targets>]...

Available positional items:
    <targets>           Specify packages or recipes to be installed

Available options:
    -s, --select        Select package interactively
    -f, --frontend=ARG  Specify the RIME frontend
    -d, --dir=ARG       Specify the directory of RIME configurations
    -h, --help          Prints help information
    -V, --version       Prints version information
```

#### Example

Install [雾凇拼音](https://github.com/iDvel/rime-ice) for fcitx5-rime:

```shell
rimeka -f fcitx5-rime iDvel/rime-ice:others/recipes/full
```
