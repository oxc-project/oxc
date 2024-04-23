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
#[diagnostic(severity(warning), help("TODO"))]
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

impl Rule for ImplementsOnClasses {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let jsdoc_finder = &ctx.jsdoc();
        let settings = &ctx.settings().jsdoc;

        // TODO: Use resolved after fixing resolver to not aliased
        let resolved_implements_tag_name = settings.resolve_tag_name("implements");
        // let resolved_class = settings.resolve_tag_name("class");
        // let resolved_constructor = settings.resolve_tag_name("constructor");

        let AstKind::Function(f) = node.kind() else {
            return;
        };
        if !f.is_function_declaration() {
            return;
        }

        // TODO: Look up parent for arrow function -> variable declaration
        let Some(jsdocs) = jsdoc_finder.get_all_by_node(node) else {
            return;
        };

        let (mut implements_found, mut class_or_ctor_found) = (0, 0);
        for tag in jsdocs.iter().flat_map(JSDoc::tags) {
            let tag_name = tag.kind.parsed();

            if tag_name == resolved_implements_tag_name {
                implements_found += 1;
            }
            if tag_name == "class" || tag_name == "constructor" {
                class_or_ctor_found += 1;
            }
        }

        if 0 < implements_found && class_or_ctor_found == 0 {
            ctx.diagnostic(ImplementsOnClassesDiagnostic(f.span));
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
			      function quux () {
			
			      }
			      ",
            None,
            None,
        ),
    ];

    Tester::new(ImplementsOnClasses::NAME, pass, fail).test_and_snapshot();
}
