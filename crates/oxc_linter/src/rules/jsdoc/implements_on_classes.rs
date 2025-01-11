use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    ast_util::is_function_node,
    context::LintContext,
    rule::Rule,
    utils::{get_function_nearest_jsdoc_node, should_ignore_as_internal, should_ignore_as_private},
    AstNode,
};

fn implements_on_classes_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`@implements` used on a non-constructor function")
        .with_help("Add `@class` tag or use ES6 class syntax.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ImplementsOnClasses;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports an issue with any non-constructor function using `@implements`.
    ///
    /// ### Why is this bad?
    ///
    /// Constructor functions should be
    /// whether marked with `@class`, `@constructs`, or being an ES6 class constructor.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// /**
    ///  * @implements {SomeClass}
    ///  */
    /// function quux () {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
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
    /// ```
    ImplementsOnClasses,
    jsdoc,
    correctness
);

fn is_function_inside_of_class<'a, 'b>(node: &'b AstNode<'a>, ctx: &'b LintContext<'a>) -> bool {
    let mut current_node = node;
    while let Some(parent_node) = ctx.nodes().parent_node(current_node.id()) {
        match parent_node.kind() {
            AstKind::MethodDefinition(_) | AstKind::PropertyDefinition(_) => return true,
            // Keep looking up only if the node is wrapped by `()`
            AstKind::ParenthesizedExpression(_) => {
                current_node = parent_node;
            }
            _ => return false,
        }
    }

    false
}

impl Rule for ImplementsOnClasses {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !is_function_node(node) {
            return;
        }

        // Filter plain declared (arrow) function.
        // I'm not sure but this rule does not care node like `MethodDefinition`.
        if is_function_inside_of_class(node, ctx) {
            return;
        }

        let Some(jsdocs) = get_function_nearest_jsdoc_node(node, ctx)
            .and_then(|node| ctx.jsdoc().get_all_by_node(node))
        else {
            return;
        };

        let settings = &ctx.settings().jsdoc;
        let resolved_implements_tag_name = settings.resolve_tag_name("implements");
        let resolved_class_tag_name = settings.resolve_tag_name("class");
        let resolved_constructor_tag_name = settings.resolve_tag_name("constructor");

        let (mut implements_found, mut class_or_ctor_found) = (None, false);
        for jsdoc in jsdocs
            .iter()
            .filter(|jsdoc| !should_ignore_as_internal(jsdoc, settings))
            .filter(|jsdoc| !should_ignore_as_private(jsdoc, settings))
        {
            for tag in jsdoc.tags() {
                let tag_name = tag.kind.parsed();

                if tag_name == resolved_implements_tag_name {
                    implements_found = Some(tag.kind.span);
                }
                if tag_name == resolved_class_tag_name || tag_name == resolved_constructor_tag_name
                {
                    class_or_ctor_found = true;
                }
            }
        }

        if let Some(span) = implements_found {
            if !class_or_ctor_found {
                ctx.diagnostic(implements_on_classes_diagnostic(span));
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

    Tester::new(ImplementsOnClasses::NAME, ImplementsOnClasses::PLUGIN, pass, fail)
        .test_and_snapshot();
}
