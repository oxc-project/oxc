//! Shared sorting engine behind import sorting and future sorting targets.
//!
//! The engine knows nothing about IR or AST node types. A target (imports today, named
//! specifiers / type members later) feeds it names, selectors, modifiers and group indices and
//! gets back orderings and permutations. Roadmap: <https://github.com/oxc-project/oxc/issues/22521>.

pub mod compare;
pub mod groups;
pub mod options;
pub mod permutation;
