use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::JSDoc;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error(
    "eslint-plugin-jsdoc(implements-on-classes): `@implements` used on a non-constructor function"
)]
#[diagnostic(severity(warning), help("Add `@class` tag or use ES6 class syntax."))]
struct ImplementsOnClassesDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct ImplementsOnClasses;

declare_oxc_lint!(
    /// ### What it does
    /// Reports an issue with any non-constructor function using `@implements`.
    ///
    /// ### Why is this bad?
    /// Constructor functions should be
    /// whether marked with `@class`, `@constructs`, or being an ES6 class constructor.
    ///
    /// ### Example
    /// ```javascript
    /// // Passing
    /// class Foo {
    ///   /**
    ///    * @implements {SomeClass}
    ///    */
    ///   constructor() {}
    /// }
    /// /**
    ///  * @implements {SomeClass}
    ///  * @class
    ///  */
    /// function quux () {}
    ///
    /// // Failing
    /// /**
    ///  * @implements {SomeClass}
    ///  */
    /// function quux () {}
    /// ```
    ImplementsOnClasses,
    correctness
);

/// Get the root node of a function.
/// ```js
/// /** FunctionDeclaration */
/// function foo() {}
///
/// /** VariableDeclaration > FunctionExpression */
/// const bar = function() {}
///
/// /** VariableDeclaration > ArrowFunctionExpression */
/// const baz = () => {}
/// ```
fn get_function_root_node<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b AstNode<'a>> {
    match node.kind() {
        AstKind::Function(f) if f.is_function_declaration() => return Some(node),
        AstKind::ArrowFunctionExpression(_) => {}
        AstKind::Function(f) if f.is_expression() => {}
        _ => return None,
    };

    let mut current_node = node;
    loop {
        match current_node.kind() {
            AstKind::Program(_) => return None,
            AstKind::VariableDeclaration(_) => return Some(current_node),
            _ => {
                current_node = ctx.nodes().parent_node(current_node.id())?;
            }
        }
    }
}

impl Rule for ImplementsOnClasses {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let settings = &ctx.settings().jsdoc;

        let resolved_implements_tag_name = settings.resolve_tag_name("implements");
        let resolved_class_tag_name = settings.resolve_tag_name("class");
        let resolved_constructor_tag_name = settings.resolve_tag_name("constructor");

        let Some(jsdocs) =
            get_function_root_node(node, ctx).and_then(|node| ctx.jsdoc().get_all_by_node(node))
        else {
            return;
        };

        let (mut implements_found, mut class_or_ctor_found) = (None, false);
        for tag in jsdocs.iter().flat_map(JSDoc::tags) {
            let tag_name = tag.kind.parsed();

            if tag_name == resolved_implements_tag_name {
                implements_found = Some(tag.kind.span);
            }
            if tag_name == resolved_class_tag_name || tag_name == resolved_constructor_tag_name {
                class_or_ctor_found = true;
            }
        }

        if let Some(span) = implements_found {
            if !class_or_ctor_found {
                ctx.diagnostic(ImplementsOnClassesDiagnostic(span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			      /**
			       * @implements {SomeClass}
			       * @class
			       */
			      function quux () {
			
			      }
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * @implements {SomeClass}
			       * @constructor
			       */
			      function quux () {
			
			      }
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * @implements {SomeClass}
			       * @constructor
			       */
			      const quux = () => {
			
			      }
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       *
			       */
			      class quux {
			        /**
			         * @implements {SomeClass}
			         */
			        constructor () {
			
			        }
			      }
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       *
			       */
			      const quux = class {
			        /**
			         * @implements {SomeClass}
			         */
			        constructor () {
			
			        }
			      }
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       *
			       */
			      class quux {
			        /**
			         * @implements {SomeClass}
			         */
			        foo() {
			
			        }
			      }
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       *
			       */
			      function quux () {
			
			      }
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * @function
			       * @implements {SomeClass}
			       */
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * @callback
			       * @implements {SomeClass}
			       */
			      ",
            None,
            None,
        ),
    ];

    let fail = vec![
        (
            "
			      /**
			       * @implements {SomeClass}
			       */
			      function quux () {
			
			      }
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * @implements {SomeClass}
			       */
			      const quux = () => {
			
			      }
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * @function
			       * @implements {SomeClass}
			       */
			      const quux = function() {
			
			      }
			      ",
            None,
            None,
        ),
    ];

    Tester::new(ImplementsOnClasses::NAME, pass, fail).test_and_snapshot();
}
