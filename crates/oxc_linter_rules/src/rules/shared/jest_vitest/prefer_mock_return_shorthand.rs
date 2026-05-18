use oxc_ast::{
    AstKind,
    ast::{
        Argument, CallExpression, Expression, IdentifierReference, ImportExpression, NewExpression,
        Statement, TaggedTemplateExpression, VariableDeclarationKind,
    },
};
use oxc_ast_visit::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::{ReferenceId, SymbolId};
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use crate::{AstNode, context::LintContext};

fn prefer_mock_return_shorthand_diagnostic(
    span: Span,
    current_property: &str,
    replacement: &str,
) -> OxcDiagnostic {
    let help = format!("Replace `{current_property}` with `{replacement}`.");

    OxcDiagnostic::warn(
        "Mock functions that return simple values should use `mockReturnValue/mockReturnValueOnce`.",
    )
    .with_help(help)
    .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

When working with mocks of functions that return simple values, Jest provides some API sugar functions to reduce the amount of boilerplate you have to write.

### Why is this bad?

Not using Jest's API sugar functions adds unnecessary boilerplate and makes tests harder to read. These helpers clearly express intent
and reduce errors, keeping tests simple and maintainable.

### Examples

Examples of **incorrect** code for this rule:
```js
jest.fn().mockImplementation(() => 'hello world');

jest
  .spyOn(fs.promises, 'readFile')
  .mockImplementationOnce(() => Promise.reject(new Error('oh noes!')));

myFunction
  .mockImplementationOnce(() => 42)
  .mockImplementationOnce(() => Promise.resolve(42))
  .mockReturnValue(0);
```

Examples of **correct** code for this rule:
```js
jest.fn().mockResolvedValue(123);

jest
  .spyOn(fs.promises, 'readFile')
  .mockReturnValue(Promise.reject(new Error('oh noes!')));
jest.spyOn(fs.promises, 'readFile').mockRejectedValue(new Error('oh noes!'));

jest.spyOn(fs, 'readFileSync').mockImplementationOnce(() => {
  throw new Error('oh noes!');
});

myFunction
  .mockResolvedValueOnce(42)
  .mockResolvedValueOnce(42)
  .mockReturnValue(0);
```
";

pub fn run<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };

    let Some(mem_expr) = call_expr.callee.as_member_expression() else {
        return;
    };

    if call_expr.arguments.is_empty() {
        return;
    }

    let Some((property_span, property_name)) = mem_expr.static_property_info() else {
        return;
    };

    let Some(expr) = call_expr.arguments.first().and_then(Argument::as_expression) else {
        return;
    };

    let is_once = property_name.ends_with("Once");

    if !property_name.eq("mockImplementation") && !property_name.eq("mockImplementationOnce") {
        return;
    }

    let Some(return_expression) = get_mock_return(expr) else {
        return;
    };

    if let Expression::UpdateExpression(_) = return_expression {
        return;
    }

    let mut visitor = IdentifierCollectorVisitor::new();

    visitor.visit_expression(return_expression);

    for reference_id in visitor.references {
        if let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id()
            && is_mutable(symbol_id, ctx)
        {
            return;
        }
    }

    let new_property_name = if is_once { "mockReturnValueOnce" } else { "mockReturnValue" };
    let diagnostic =
        prefer_mock_return_shorthand_diagnostic(property_span, property_name, new_property_name);

    if contains_call_like_expression(return_expression) {
        ctx.diagnostic(diagnostic);
        return;
    }

    ctx.diagnostic_with_fix(diagnostic, |fixer| {
        let return_text =
            ctx.source_range(GetSpan::span(return_expression.without_parentheses())).to_owned();
        let argument_span = GetSpan::span(expr);

        let mut multifixer = fixer.for_multifix().new_fix_with_capacity(2);

        multifixer.push(fixer.replace(property_span, new_property_name));
        multifixer.push(fixer.replace(argument_span, return_text));

        multifixer.with_message("Replaced successfully")
    });
}

fn is_mutable(symbol_id: SymbolId, ctx: &LintContext<'_>) -> bool {
    let scoping = ctx.scoping();

    if scoping.symbol_is_mutated(symbol_id) {
        return true;
    }

    let decl_node_id = scoping.symbol_declaration(symbol_id);
    if let AstKind::VariableDeclarator(_) = ctx.nodes().kind(decl_node_id)
        && let AstKind::VariableDeclaration(parent) = ctx.nodes().parent_kind(decl_node_id)
    {
        return parent.kind != VariableDeclarationKind::Const;
    }

    false
}

fn get_mock_return<'a>(argument_expression: &'a Expression<'a>) -> Option<&'a Expression<'a>> {
    match argument_expression {
        Expression::ArrowFunctionExpression(arrow_func) => {
            if arrow_func.r#async
                || arrow_func.body.statements.len() > 1
                || arrow_func.params.has_parameter()
            {
                return None;
            }

            let stmt = arrow_func.body.statements.first()?;

            match stmt {
                Statement::ExpressionStatement(stmt_expr) => Some(&stmt_expr.expression),
                Statement::ReturnStatement(return_statement) => {
                    let Some(arg_expr) = &return_statement.argument else {
                        return None;
                    };

                    Some(arg_expr)
                }
                _ => None,
            }
        }
        Expression::FunctionExpression(function) => {
            if function.r#async || function.params.has_parameter() {
                return None;
            }

            let Some(body) = &function.body else {
                return None;
            };

            if body.statements.len() > 1 {
                return None;
            }

            let stmt = body.statements.first()?;

            match stmt {
                Statement::ExpressionStatement(stmt_expr) => Some(&stmt_expr.expression),
                Statement::ReturnStatement(return_statement) => {
                    let Some(arg_expr) = &return_statement.argument else {
                        return None;
                    };

                    Some(arg_expr)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn contains_call_like_expression(expr: &Expression<'_>) -> bool {
    let mut visitor = CallLikeExpressionVisitor::default();
    visitor.visit_expression(expr);
    visitor.contains_call_like_expression
}

#[derive(Default)]
struct CallLikeExpressionVisitor {
    contains_call_like_expression: bool,
}

impl<'a> Visit<'a> for CallLikeExpressionVisitor {
    fn visit_call_expression(&mut self, _it: &CallExpression<'a>) {
        self.contains_call_like_expression = true;
    }

    fn visit_new_expression(&mut self, _it: &NewExpression<'a>) {
        self.contains_call_like_expression = true;
    }

    fn visit_tagged_template_expression(&mut self, _it: &TaggedTemplateExpression<'a>) {
        self.contains_call_like_expression = true;
    }

    fn visit_import_expression(&mut self, _it: &ImportExpression<'a>) {
        self.contains_call_like_expression = true;
    }
}

struct IdentifierCollectorVisitor {
    references: FxHashSet<ReferenceId>,
}

impl IdentifierCollectorVisitor {
    fn new() -> Self {
        Self { references: FxHashSet::default() }
    }
}

impl<'a> Visit<'a> for IdentifierCollectorVisitor {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        self.references.insert(ident.reference_id());
    }
}
