# Contributor License Agreement

Please note that you will be required to sign the [Contributor License Agreement](https://cla-assistant.io/web-infra-dev/oxc) before your pull request can be accepted.

# Installation

## Clone Repo

```bash
git clone --recurse-submodules git@github.com:web-infra-dev/oxc.git
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

Some additional Cargo tools are required for developing this project, they can be installed via [`cargo binstall`](https://github.com/cargo-bins/cargo-binstall). `cargo binstall` provides a low-complexity mechanism for installing rust binaries as an alternative to building from source via the slower `cargo install`.

You can download the [pre-compiled binary](https://github.com/cargo-bins/cargo-binstall#installation) and save it in `~/.cargo/bin` or install it by running `cargo install cargo-binstall`


## Required tools

```bash
cargo binstall just -y
```

[`just`](https://github.com/casey/just) is a handy way to save and run project-specific commands.
To initialize all the required tools, run

```
just init
```

## Commands

Run `just` for the list of available commands.

Run `just r` (alias for `just ready`) to make sure the whole project builds and runs correctly.

Take a look at `just new-rule` if you need to start porting a new ESLint rule.
Make sure the rule is registered in `crates/oxc_linter/src/rules.rs`.

# Performance Turing

## Mac Xcode Instruments

Mac Xcode instruments can be used to produce a CPU profile.

To install Xcode Instruments, install the Command Line Tools:

```bash
xcode-select --install
```

For normal Rust builds, [`cargo instruments`](https://github.com/cmyr/cargo-instruments) can be used as the glue
for profiling and creating the trace file.

First, change the profile for showing debug symbols.

```toml
[profile.release]
debug = 1 # debug info with line tables only
strip = false # do not strip symbols
```

Then build the project

```bash
cargo build --release -p oxc_cli --bin oxlint
```

The binary is located at `./target/release/oxlint` once the project is built.

Under the hood, `cargo instruments` invokes the `xcrun` command, equivalent to

```bash
xcrun xctrace record --template 'Time Profile' --output . --launch -- /path/to/oxc/target/release/oxlint --quiet .
```

Running the command above produces the following output

```
Starting recording with the Time Profiler template. Launching process: oxlint.
Ctrl-C to stop the recording
Target app exited, ending recording...
Recording completed. Saving output file...
Output file saved as: Launch_oxlint_2023-09-03_4.41.45 PM_EB179B85.trace
```

Open the trace file `open Launch_oxlint_2023-09-03_4.41.45\ PM_EB179B85.trace`.

To see a top down trace:

1. On the top panel, click CPUs
2. On the left input box, click `x` then select `Time Profiler`
3. At the bottom panel, click "Call Tree", turn on "Invert Call Tree" and turn off separate by thread.
