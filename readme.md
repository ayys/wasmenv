# wasmenv

`wasmenv` is a Rust-based version manager for wasm runtimes.  Right now wasmtime supports `wasmer` only.
This project allows you to easily switch between different versions of `wasmer`, manage installations, and configure your shell environment.

## Installation

To install `wasmenv` from crates.io:
```shell
cargo install wasmenv
```

To develop `wasmenv`, clone the repository and build it using Cargo:
```shell
git clone https://github.com/ayys/wasmenv.git
cd wasmenv
cargo build --release
export PATH=$PATH:$PWD/target/release
```

## Usage

```shell
$ wasmenv --help

Usage: wasmenv <COMMAND>

Commands:
  current  Display the currently active version of wasmer
  shell    Configure wasmenv for a specific shell (bash, zsh, fish)
  use      Install wasmer
  list     List all the available versions of wasmer
  exec     Run command with wasmer
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

_Please make sure to update tests as appropriate._


## License

[MIT](https://choosealicense.com/licenses/mit/)
