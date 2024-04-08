use serde::Deserialize;

use crate::options::{default_as_true, JsxOptions};

/// [preset-typescript](https://babeljs.io/docs/babel-preset-typescript#options)
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TypeScriptOptions {
    #[serde(default = "default_as_true")]
    pub allow_namespaces: bool,

    pub only_remove_type_imports: bool,

    pub optimize_const_enums: bool,

    pub rewrite_import_extensions: bool,
}

impl Default for TypeScriptOptions {
    fn default() -> Self {
        Self {
            allow_namespaces: default_as_true(),
            only_remove_type_imports: false,
            optimize_const_enums: false,
            rewrite_import_extensions: false,
        }
    }
}

/// [plugin-transform-typescript](https://babeljs.io/docs/babel-plugin-transform-typescript)
///
/// This plugin adds support for the types syntax used by the TypeScript programming language.
/// However, this plugin does not add the ability to type-check the JavaScript passed to it.
/// For that, you will need to install and set up TypeScript.
///
/// Note that although the TypeScript compiler tsc actively supports certain JavaScript proposals such as optional chaining (?.),
/// nullish coalescing (??) and class properties (this.#x), this preset does not include these features
/// because they are not the types syntax available in TypeScript only.
/// We recommend using preset-env with preset-typescript if you want to transpile these features.
///
/// This plugin is included in `preset-typescript`.
///
/// ## Example
///
/// In:  `const x: number = 0;`
/// Out: `const x = 0;`
#[allow(dead_code)]
pub struct TypeScript {
    options: TypeScriptOptions,
    jsx: JsxOptions,
}

impl TypeScript {
    #[allow(unused)]
    pub fn new(options: TypeScriptOptions, jsx: JsxOptions) -> Self {
        Self { options, jsx }
    }
}
