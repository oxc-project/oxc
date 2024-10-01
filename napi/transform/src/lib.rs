// NOTE: the strange order of struct and `mod` statements is to establish the
// desired order in generated `index.d.ts` code. We want options to be on top.
// This is not only for aesthetics, but using declarations before they're parsed
// breaks NAPI typegen.
mod context;
mod options;

pub use crate::options::*;

mod sourcemap;
pub use crate::sourcemap::*;

mod isolated_declaration;
pub use isolated_declaration::*;

mod transformer;
pub use transformer::*;
