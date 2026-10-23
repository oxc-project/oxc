//! Type-aware lint rules for oxlint: a port of tsgolint (oxc-project/tsgolint, MIT) on tsrs, the Rust port of
//! the TypeScript 7 checker. `oxc_linter`'s `tsrs` feature links this crate and runs it in-process through
//! `linter::spawn`; `protocol` keeps tsgolint's headless payload and message types, which `oxc_linter` already
//! speaks, so the rules, the file -> tsconfig assignment and the programs are the same as with the `tsgolint`
//! subprocess. The port came from tsrslint, a standalone drop-in `tsgolint` binary.

#[cfg(target_os = "macos")]
mod boundedfs;
pub mod linter;
mod memstats;
pub mod overlayfs;
pub mod protocol;
pub mod rule;
pub mod rules;
mod sched;
pub mod tsconfig;
pub mod utils;
