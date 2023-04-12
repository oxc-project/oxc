# Installation

## Clone Repo

```bash
git clone --recurse-submodules git@github.com:Boshen/oxc.git
```

## New to Rust

### Install Rust
```bash
# https://rustup.rs/
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

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

* `just` is a handy way to save and run project-specific commands

To initialize all the required tools, run

```
just init
```

This installs:

* `cargo nextest`, a faster test runner compared to `cargo test`
* `cargo watch`, for watching and running commands
* `cargo insta`, for snapshot testing

## Commands

Run `just` for the list of available commands.
