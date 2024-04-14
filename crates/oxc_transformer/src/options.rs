use crate::{
    compiler_assumptions::CompilerAssumptions, decorators::DecoratorsOptions, react::ReactOptions,
    typescript::TypeScriptOptions,
};

#[derive(Debug, Default, Clone)]
pub struct TransformOptions {
    // Core
    /// Set assumptions in order to produce smaller output.
    /// For more information, check the [assumptions](https://babel.dev/docs/assumptions) documentation page.
    pub assumptions: CompilerAssumptions,

    // Plugins
    /// [proposal-decorators](https://babeljs.io/docs/babel-plugin-proposal-decorators)
    pub decorators: DecoratorsOptions,

    /// [preset-typescript](https://babeljs.io/docs/babel-preset-typescript)
    pub typescript: TypeScriptOptions,

    /// [preset-react](https://babeljs.io/docs/babel-preset-react)
    pub react: ReactOptions,
}
