use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::Symbol;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-shadow-restricted-names): Shadowing of global property '{0}'")]
#[diagnostic(severity(warning))]
struct NoShadowRestrictedNamesDiagnostic(
    Atom,
    #[label("Shadowing of global property '{0}'")] pub Span,
);

#[derive(Debug, Default, Clone)]
pub struct NoShadowRestrictedNames;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow identifiers from shadowing restricted names
    ///
    /// ### Why is this bad?
    /// ES5 §15.1.1 Value Properties of the Global Object (NaN, Infinity, undefined) as well as strict mode restricted identifiers eval and arguments are considered to be restricted names in JavaScript.
    /// Defining them to mean something else can have unintended consequences and confuse others reading the code.
    ///
    /// ### Example
    /// ```javascript
    /// var undefined = "foo";
    /// ```
    NoShadowRestrictedNames,
    correctness
);

static RESTRICTED: [&str; 5] = ["undefined", "NaN", "Infinity", "arguments", "eval"];

fn safely_shadows_undefined(symbol: &Symbol, ctx: &LintContext<'_>) -> bool {
    if symbol.name().as_str() == "undefined" {
        let mut no_assign = true;
        let symbols = ctx.symbols();
        for reference_id in symbol.references() {
            let reference = symbols.get_resolved_reference(*reference_id).unwrap();
            if reference.is_write() {
                no_assign = false;
            }
        }
        let mut no_init = false;
        let decl = ctx.nodes()[symbol.declaration()];
        if let AstKind::VariableDeclarator(var) = decl.kind() {
            no_init = var.init.is_none();
        }
        return no_assign && no_init;
    }
    false
}

impl Rule for NoShadowRestrictedNames {
    fn run_on_symbol(&self, symbol: &Symbol, ctx: &LintContext<'_>) {
        if RESTRICTED.contains(&symbol.name().as_str()) && !safely_shadows_undefined(symbol, ctx) {
            ctx.diagnostic(NoShadowRestrictedNamesDiagnostic(symbol.name().clone(), symbol.span()));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function foo(bar){ var baz; }", None),
        ("!function foo(bar){ var baz; }", None),
        ("!function(bar){ var baz; }", None),
        ("try {} catch(e) {}", None),
        ("export default function() {}", None),
        ("var undefined;", None),
        ("var undefined; doSomething(undefined);", None),
        ("var undefined; var undefined;", None),
        ("let undefined", None),
    ];

    let fail = vec![
        ("function NaN(NaN) { var NaN; !function NaN(NaN) { try {} catch(NaN) {} }; }", None),
        (
            "function undefined(undefined) { !function undefined(undefined) { try {} catch(undefined) {} }; }",
            None,
        ),
        (
            "function Infinity(Infinity) { var Infinity; !function Infinity(Infinity) { try {} catch(Infinity) {} }; }",
            None,
        ),
        (
            "function arguments(arguments) { var arguments; !function arguments(arguments) { try {} catch(arguments) {} }; }",
            None,
        ),
        ("function eval(eval) { var eval; !function eval(eval) { try {} catch(eval) {} }; }", None),
        (
            "var eval = (eval) => { var eval; !function eval(eval) { try {} catch(eval) {} }; }",
            None,
        ),
        ("var [undefined] = [1]", None),
        (
            "var {undefined} = obj; var {a: undefined} = obj; var {a: {b: {undefined}}} = obj; var {a, ...undefined} = obj;",
            None,
        ),
        ("var undefined; undefined = 5;", None),
    ];

    Tester::new(NoShadowRestrictedNames::NAME, pass, fail).test_and_snapshot();
}
