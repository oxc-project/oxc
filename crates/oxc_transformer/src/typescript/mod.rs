mod namespace;

use std::rc::Rc;

use serde::Deserialize;

use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::context::Ctx;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptOptions;

/// [Preset TypeScript](https://babeljs.io/docs/babel-preset-typescript)
///
/// This preset includes the following plugins:
///
/// * [transform-typescript](https://babeljs.io/docs/babel-plugin-transform-typescript)
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
#[allow(unused)]
pub struct TypeScript<'a> {
    options: TypeScriptOptions,
    ctx: Ctx<'a>,
}

impl<'a> TypeScript<'a> {
    pub fn new(options: TypeScriptOptions, ctx: &Ctx<'a>) -> Self {
        Self { options, ctx: Rc::clone(ctx) }
    }
}

// Transforms
impl<'a> TypeScript<'a> {
    pub fn transform_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        self.transform_statements_for_namespace(stmts);
    }
}
