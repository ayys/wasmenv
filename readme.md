# wasmerenv

## TODO
- [ ] Add unit tests, right now there are no tests at all.
- [ ] Add integration tests with wasmer cli (run a wasm file with different versions of wasmer)
- [ ] Add CI to publish to crates.io

`wasmerenv` is a Rust-based version manager for Wasmer, a WebAssembly runtime. This project allows you to easily switch between different versions of Wasmer, manage installations, and configure your shell environment.

## Installation

To install `wasmerenv` from crates.io:
```shell
cargo install wasmerenv # Not possible ATM because I am not publishing to crates.io right now
```

To install `wasmerenv`, clone the repository and build it using Cargo:
```shell
git clone https://github.com/yourusername/wasmerenv.git
cd wasmerenv
cargo build --release
export PATH=$PATH:$PWD/target/release
```

## Usage

```shell
$ wasmerenv --help

Usage: wasmerenv <COMMAND>

Commands:
  current  Display the currently active version of wasmer
  shell    Configure wasmerenv for a specific shell (bash, zsh, fish)
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
