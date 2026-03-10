use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn runtime_localize_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`$localize` used at module load time")
        .with_help(
            "Move the `$localize` call inside a function, method, or getter to ensure it runs \
            at runtime when localization data is available. Using `$localize` at module load time \
            may cause issues if the localization data hasn't been loaded yet.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RuntimeLocalize;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows usage of `$localize` at module load time (top-level code).
    ///
    /// ### Why is this bad?
    ///
    /// The `$localize` template tag function requires localization data to be loaded before
    /// it can properly translate strings. When `$localize` is used at module load time
    /// (e.g., in top-level variable declarations, class field initializers, or static properties),
    /// the localization data may not yet be available, resulting in:
    /// - Untranslated strings
    /// - Missing translations
    /// - Runtime errors in some configurations
    ///
    /// Instead, use `$localize` inside functions, methods, or getters that are called at runtime
    /// after the localization data has been loaded.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// // Top-level constant
    /// const GREETING = $localize`Hello`;
    ///
    /// // Static class property
    /// class MyComponent {
    ///   static readonly TITLE = $localize`Welcome`;
    /// }
    ///
    /// // Class field initializer
    /// class MyComponent {
    ///   message = $localize`Hello World`;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// // Inside a method
    /// class MyComponent {
    ///   getGreeting() {
    ///     return $localize`Hello`;
    ///   }
    /// }
    ///
    /// // Inside a getter
    /// class MyComponent {
    ///   get title() {
    ///     return $localize`Welcome`;
    ///   }
    /// }
    ///
    /// // Inside ngOnInit
    /// class MyComponent {
    ///   message: string;
    ///   ngOnInit() {
    ///     this.message = $localize`Hello World`;
    ///   }
    /// }
    /// ```
    RuntimeLocalize,
    angular,
    pedantic,
    pending
);

impl Rule for RuntimeLocalize {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TaggedTemplateExpression(tagged) = node.kind() else {
            return;
        };

        // Check if the tag is $localize
        let tag_name = match &tagged.tag {
            oxc_ast::ast::Expression::Identifier(ident) => ident.name.as_str(),
            _ => return,
        };

        if tag_name != "$localize" {
            return;
        }

        // Check if we're at module load time
        if is_at_module_load_time(node, ctx) {
            ctx.diagnostic(runtime_localize_diagnostic(tagged.span));
        }
    }
}

fn is_at_module_load_time(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    // Walk up the tree to check the context
    for ancestor in ctx.nodes().ancestors(node.id()) {
        match ancestor.kind() {
            // Inside a function/method/arrow function = runtime
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                return false;
            }
            // Inside a getter = runtime
            AstKind::MethodDefinition(method) => {
                if method.kind.is_get() {
                    return false;
                }
                // Regular method = runtime
                return false;
            }
            // Inside a property definition = module load time
            AstKind::PropertyDefinition(prop) => {
                // Static properties are especially problematic
                if prop.r#static {
                    return true;
                }
                // Instance properties are also module load time
                return true;
            }
            // Program level = module load time
            AstKind::Program(_) => {
                return true;
            }
            _ => {}
        }
    }

    // If we reach here without finding a function context, it's likely module load time
    true
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Inside a method
        r"
        class TestComponent {
            getGreeting() {
                return $localize`Hello`;
            }
        }
        ",
        // Inside ngOnInit
        r"
        class TestComponent {
            ngOnInit() {
                this.message = $localize`Hello World`;
            }
        }
        ",
        // Inside a getter
        r"
        class TestComponent {
            get title() {
                return $localize`Welcome`;
            }
        }
        ",
        // Inside constructor
        r"
        class TestComponent {
            constructor() {
                this.message = $localize`Hello`;
            }
        }
        ",
        // Inside an arrow function that's called later
        r"
        class TestComponent {
            getMessage = () => $localize`Hello`;
        }
        ",
        // Not $localize
        r"
        const value = someOtherTag`Hello`;
        ",
    ];

    let fail = vec![
        // Top-level constant
        r"
        const GREETING = $localize`Hello`;
        ",
        // Static class property
        r"
        class MyComponent {
            static readonly TITLE = $localize`Welcome`;
        }
        ",
        // Class field initializer
        r"
        class MyComponent {
            message = $localize`Hello World`;
        }
        ",
        // Top-level export
        r"
        export const ERROR_MESSAGE = $localize`An error occurred`;
        ",
    ];

    Tester::new(RuntimeLocalize::NAME, RuntimeLocalize::PLUGIN, pass, fail).test_and_snapshot();
}
