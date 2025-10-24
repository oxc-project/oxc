use oxc_ast::AstKind;
use oxc_ast::ast::{
    CallExpression, Class, Expression, ObjectExpression, ObjectProperty, ObjectPropertyKind,
    PropertyKey,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_standalone_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer standalone components.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferStandalone;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Components, Directives and Pipes should not opt out of standalone.
    ///
    /// ### Why is this bad?
    ///
    /// Since Angular 19, components, directives and pipes have been standalone by default.
    /// It is the recommended way to create them.
    ///  Therefore, you should not opt out of standalone.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// @Component({ standalone: false })
    /// class TestComponent {}
    ///
    /// @Directive({ standalone: false })
    /// class TestDirective {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// @Component()
    /// class TestComponent {}
    ///
    /// @Component({ standalone: true })
    /// class TestComponent {}
    ///
    /// @Directive()
    /// class TestDirective {}
    ///
    /// @Directive({ standalone: true })
    /// class TestDirective {}
    /// ```
    PreferStandalone,
    angular,
    style,
    suggestion
);

impl Rule for PreferStandalone {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Class(class) = node.kind() else {
            return;
        };
        let Some(call_expression) = get_relevant_decorator_call_expression(class) else { return };
        let Some(first_arg) = call_expression.arguments.first() else {
            return;
        };
        let Expression::ObjectExpression(obj_expression) = &first_arg.to_expression() else {
            return;
        };
        let Some(prop) = get_standalone_property(obj_expression) else { return };
        let Expression::BooleanLiteral(bool_value) = &prop.value else {
            return;
        };

        if !bool_value.value {
            ctx.diagnostic_with_suggestion(prefer_standalone_diagnostic(prop.span()), |fixer| {
                fixer.replace(prop.span(), "")
            });
        }
    }
}

fn get_relevant_decorator_call_expression<'a>(
    class: &'a Class<'a>,
) -> Option<&'a CallExpression<'a>> {
    for decorator in &class.decorators {
        let Expression::CallExpression(call_expr) = &decorator.expression else {
            continue;
        };
        let Some(callee_identifier) = call_expr.callee.get_identifier_reference() else { continue };
        if ["Component", "Directive", "Pipe"].contains(&callee_identifier.name.as_str()) {
            return Some(call_expr);
        }
    }
    None
}

fn get_standalone_property<'a>(
    decorator_object_expression: &'a ObjectExpression<'a>,
) -> Option<&'a ObjectProperty<'a>> {
    for property in &decorator_object_expression.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = &property else {
            continue;
        };
        let PropertyKey::StaticIdentifier(ident) = &prop.key else {
            continue;
        };
        if ident.name == "standalone" {
            return Some(prop);
        }
    }
    None
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "@Component({})
        class Test {}",
        "@Component({
            standalone: true,
        })
        class Test {}",
        "@Component({
            selector: 'test-selector'
        })
        class Test {}",
        "@Component({
            standalone: true,
            selector: 'test-selector'
        })
        class Test {}",
        "@Component({
            selector: 'test-selector',
            template: '<div></div>',
            styleUrls: ['./test.css']
        })
        class Test {}",
        "@Component({
            selector: 'test-selector',
            standalone: true,
            template: '<div></div>',
            styleUrls: ['./test.css']
        })
        class Test {}",
        "@Directive({})
        class Test {}",
        "@Directive({
            standalone: true,
        })
        class Test {}",
        "@Directive({
            selector: 'test-selector'
        })
        class Test {}",
        "@Directive({
            standalone: true,
            selector: 'test-selector'
        })
        class Test {}",
        "@Directive({
            selector: 'test-selector',
            providers: []
        })
        class Test {}",
        "@Directive({
            selector: 'test-selector',
            standalone: true,
            providers: []
        })
        class Test {}",
        "@Directive()
        abstract class Test {}",
        "@Pipe({})
        class Test {}",
        "@Pipe({
            standalone: true,
        })
        class Test {}",
        "@Pipe({
            name: 'test-pipe'
        })
        class Test {}",
        "@Pipe({
            standalone: true,
            name: 'test-pipe'
        })
        class Test {}",
        "@Pipe({
            name: 'my-pipe',
            pure: true
        })
        class Test {}",
        "@Pipe({
            name: 'my-pipe',
            standalone: true,
            pure: true
        })
        class Test {}",
    ];

    let fail = vec![
        "@Component({ standalone: false })
        class Test {}",
        "@Component({
            standalone: false,
            template: '<div></div>'
        })
        class Test {}",
        "@Directive({ standalone: false })
        class Test {}",
        "@Directive({
            standalone: false,
            selector: 'x-selector'
        })
        class Test {}",
        "@Pipe({ standalone: false })
        class Test {}",
        "@Pipe({
            standalone: false,
            name: 'pipe-name'
        })
        class Test {}",
    ];

    let fix = vec![
        (
            "@Component({ standalone: false })
            class Test {}",
            "@Component({  })
            class Test {}",
        ),
        (
            "@Component({
                standalone: false,
                template: '<div></div>'
            })
            class Test {}",
            "@Component({
                ,
                template: '<div></div>'
            })
            class Test {}",
        ),
        (
            "@Directive({ standalone: false })
            class Test {}",
            "@Directive({  })
            class Test {}",
        ),
        (
            "@Directive({
                standalone: false,
                selector: 'x-selector'
            })
            class Test {}",
            "@Directive({
                ,
                selector: 'x-selector'
            })
            class Test {}",
        ),
        (
            "@Pipe({ standalone: false })
            class Test {}",
            "@Pipe({  })
            class Test {}",
        ),
        (
            "@Pipe({
                standalone: false,
                name: 'pipe-name'
            })
            class Test {}",
            "@Pipe({
                ,
                name: 'pipe-name'
            })
            class Test {}",
        ),
    ];
    Tester::new(PreferStandalone::NAME, PreferStandalone::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
