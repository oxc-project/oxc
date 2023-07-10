# Installation

## Clone Repo

```bash
git clone --recurse-submodules git@github.com:Boshen/oxc.git
```

The `--recurse-submodules` flag will install the following submodules:
- [babel](https://github.com/babel/babel) registered for path `tasks/coverage/babel`
- [test262](https://github.com/tc39/test262) registered for path `tasks/coverage/test262`
- [typescript](https://github.com/microsoft/TypeScript) registered for path `tasks/coverage/typescript`

## New to Rust

### Install Rust
```bash
# https://rustup.rs/
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

```bash
# move to the cloned repository
cd oxc
```

```bash
# rust toolchain
rustup show
```

`rustup show` reads the `./rust-toolchain.toml` file and installs the correct Rust toolchain and components for this project.

### Cargo Tools

Additional Rust tools can be installed by `cargo install`. They are stored under `~/.cargo/bin`.

[`cargo binstall`](https://github.com/cargo-bins/cargo-binstall) provides a low-complexity mechanism for installing rust binaries as an alternative to building from source via the slower `cargo install`.

You can download the [pre-compiled binary](https://github.com/cargo-bins/cargo-binstall#installation) or install via `cargo install cargo-binstall`


## Required tools

```bash
cargo binstall just -y
```

[`just`](https://github.com/casey/just) is a handy way to save and run
project-specific commands. To initialize all the required tools, run

```
just init
```

## Commands

Run `just` for the list of available commands.

Checkout `just new-rule` if you need to start writing a new linter rule from ESLint.
