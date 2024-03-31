//! Transformer / Transpiler
//!
//! References:
//! * <https://www.typescriptlang.org/tsconfig#target>
//! * <https://babel.dev/docs/presets>
//! * <https://github.com/microsoft/TypeScript/blob/main/src/compiler/transformer.ts>

// Plugins: <https://babeljs.io/docs/plugins-list>
mod decorators;
mod react_display_name;
mod react_jsx;
mod react_jsx_self;
mod react_jsx_source;
mod typescript;

#[cfg(test)]
use oxc_ast::{AstNode, Visit};
#[cfg(test)]
use oxc_parser::Parser;
#[cfg(test)]
use oxc_semantic::AstNodeId;
#[cfg(test)]
use oxc_semantic::SemanticBuilder;
#[cfg(test)]
use std::path::PathBuf;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_diagnostics::Error;
use oxc_semantic::Semantic;
use oxc_span::SourceType;

pub use crate::{
    decorators::{Decorators, DecoratorsOptions},
    react_display_name::{ReactDisplayName, ReactDisplayNameOptions},
    react_jsx::{ReactJsx, ReactJsxOptions},
    react_jsx_self::{ReactJsxSelf, ReactJsxSelfOptions},
    react_jsx_source::{ReactJsxSource, ReactJsxSourceOptions},
    typescript::{TypeScript, TypeScriptOptions},
};

#[allow(unused)]
#[derive(Debug, Default, Clone)]
pub struct TransformOptions {
    pub decorators: DecoratorsOptions,
    pub typescript: TypeScriptOptions,
    pub react_jsx: ReactJsxOptions,
    pub react_display_name: ReactDisplayNameOptions,
    pub react_jsx_self: ReactJsxSelfOptions,
    pub react_jsx_source: ReactJsxSourceOptions,
}

#[allow(unused)]
pub struct Transformer<'a> {
    allocator: &'a Allocator,
    source_type: SourceType,
    semantic: Semantic<'a>,
    options: TransformOptions,

    // Stage 3
    decorators: Decorators,
    // [preset-typescript](https://babeljs.io/docs/babel-preset-typescript)
    typescript: TypeScript,
    // [preset-react](https://babeljs.io/docs/babel-preset-react)
    react_display_name: ReactDisplayName,
    react_jsx: ReactJsx,
    react_jsx_self: ReactJsxSelf,
    react_jsx_source: ReactJsxSource,
}

impl<'a> Transformer<'a> {
    pub fn new(
        allocator: &'a Allocator,
        source_type: SourceType,
        semantic: Semantic<'a>,
        options: TransformOptions,
    ) -> Self {
        Self {
            allocator,
            source_type,
            semantic,
            options,
            decorators: Decorators::default(),
            typescript: TypeScript::default(),
            react_display_name: ReactDisplayName::default(),
            react_jsx: ReactJsx::default(),
            react_jsx_self: ReactJsxSelf::default(),
            react_jsx_source: ReactJsxSource::default(),
        }
    }

    /// # Errors
    ///
    /// Returns `Vec<Error>` if any errors were collected during the transformation.
    pub fn build(self, _program: &mut Program<'a>) -> Result<(), Vec<Error>> {
        Ok(())
    }
}

#[cfg(test)]
struct Visitor<'a>(Semantic<'a>);

#[cfg(test)]
impl<'a> Visit<'a> for Visitor<'a> {
    fn visit_jsx_member_expression_object(
        &mut self,
        expr: &oxc_ast::ast::JSXMemberExpressionObject<'a>,
    ) {
        let id = expr.ast_node_id().unwrap();

        assert_eq!(id, AstNodeId::new(6));
        assert_eq!(self.0.nodes().ancestors(id).collect::<Vec<_>>(), vec![6, 5, 4, 3, 2, 1, 0]);
    }
}

#[test]
fn manual_test() {
    let source_text = "<foo.bar.baz>{expr}</>";
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_jsx(true);

    let ret = Parser::new(&allocator, &source_text, source_type).parse();
    if !ret.errors.is_empty() {
        for error in ret.errors {
            let error = error.with_source_code(source_text);
            println!("{error:?}");
        }
        return;
    }
    println!("Original:\n");
    println!("{source_text}\n");

    let semantic = SemanticBuilder::new(&source_text, source_type)
        .with_trivias(ret.trivias)
        .build_module_record(PathBuf::new(), &ret.program)
        .build(&ret.program)
        .semantic;

    let program = allocator.alloc(ret.program);

    let mut visitor = Visitor(semantic);
    visitor.visit_program(&program);
}
