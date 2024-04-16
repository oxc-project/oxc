use std::path::PathBuf;

use crate::{
    compiler_assumptions::CompilerAssumptions, react::ReactOptions, typescript::TypeScriptOptions,
};

/// <https://babel.dev/docs/options>
#[derive(Debug, Default, Clone)]
pub struct TransformOptions {
    //
    // Primary Options
    //
    /// The working directory that all paths in the programmatic options will be resolved relative to.
    pub cwd: PathBuf,

    // Core
    /// Set assumptions in order to produce smaller output.
    /// For more information, check the [assumptions](https://babel.dev/docs/assumptions) documentation page.
    pub assumptions: CompilerAssumptions,

    // Plugins
    /// [preset-typescript](https://babeljs.io/docs/babel-preset-typescript)
    pub typescript: TypeScriptOptions,

    /// [preset-react](https://babeljs.io/docs/babel-preset-react)
    pub react: ReactOptions,
}
