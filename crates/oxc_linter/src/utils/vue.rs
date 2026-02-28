use crate::LintContext;
use oxc_allocator::Vec;
use oxc_ast::{
    AstKind,
    ast::{
        CallExpression, ExportDefaultDeclarationKind, Expression, JSXChild, ObjectPropertyKind,
        Statement,
    },
};
use oxc_semantic::ScopeId;
use oxc_span::{GetSpan, Span};

/// Check if any of the other contexts has a default export with the `name` property.
///
/// # Example
///
/// ```vue
/// <script>
/// export default {
///  emits: []
/// }
/// </script>
/// ```
///
/// Check if it has `emits` property with `has_default_exports_property(others, "emits")`
pub fn has_default_exports_property(ctx: &LintContext<'_>, check_name: &str) -> bool {
    for other_node in ctx.semantic().nodes() {
        let AstKind::ExportDefaultDeclaration(export) = other_node.kind() else {
            continue;
        };

        let ExportDefaultDeclarationKind::ObjectExpression(export_obj) = &export.declaration else {
            continue;
        };

        let has_emits_exports = export_obj.properties.iter().any(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return false;
            };

            property.key.name().is_some_and(|name| name == check_name)
        });

        if has_emits_exports {
            return true;
        }
    }

    false
}

pub enum DefineMacroProblem {
    DefineInBoth,
    HasTypeAndArguments,
    EventsNotDefined,
    ReferencingLocally,
}

pub fn check_define_macro_call_expression(
    call_expr: &CallExpression,
    ctx: &LintContext,
    has_export_default_equivalent: bool,
) -> Option<DefineMacroProblem> {
    let has_type_args = call_expr.type_arguments.is_some();

    if has_type_args && has_export_default_equivalent {
        return Some(DefineMacroProblem::DefineInBoth);
    }

    // `defineEmits` has type arguments and js arguments. Vue Compiler allows only one of them.
    if has_type_args && !call_expr.arguments.is_empty() {
        return Some(DefineMacroProblem::HasTypeAndArguments);
    }

    if has_type_args {
        // If there are type arguments, we don't need to check the arguments.
        return None;
    }

    let Some(expression) = call_expr.arguments.first().and_then(|first| first.as_expression())
    else {
        // `defineEmits();` is valid when `export default { emits: [] }` is defined
        if !has_export_default_equivalent {
            return Some(DefineMacroProblem::EventsNotDefined);
        }
        return None;
    };

    if has_export_default_equivalent {
        return Some(DefineMacroProblem::DefineInBoth);
    }

    match expression {
        Expression::ArrayExpression(_) | Expression::ObjectExpression(_) => None,
        Expression::Identifier(identifier) => {
            // Check if the identifier is defined in the `<script setup>` block
            if ctx.scoping().get_binding(get_vue_setup_scope_id(ctx), identifier.name).is_some() {
                Some(DefineMacroProblem::ReferencingLocally)
            } else {
                None
            }
        }
        _ => Some(DefineMacroProblem::EventsNotDefined),
    }
}

/// According to <https://github.com/liangmiQwQ/vue-oxc-toolkit/blob/main/MAPPING.md>,
/// In vue-oxc-toolkit compiled AST, the last top-level statement contains
/// the `<script setup>` code.
pub fn get_vue_setup_statements<'a>(ctx: &LintContext<'a>) -> &'a [Statement<'a>] {
    let program = ctx.nodes().program();

    // Find the last top-level ArrowFunctionExpression
    let Statement::ExpressionStatement(block) = program.body.last().unwrap() else {
        unreachable!();
    };

    let Expression::ArrowFunctionExpression(function) = &block.expression else {
        unreachable!();
    };

    &function.body.statements
}

/// Get the scope ID of the Vue setup block (last top-level ArrowFunctionExpression).
pub fn get_vue_setup_scope_id(ctx: &LintContext<'_>) -> ScopeId {
    let program = ctx.nodes().program();

    // Find the last top-level ArrowFunctionExpression
    let Statement::ExpressionStatement(block) = program.body.last().unwrap() else {
        unreachable!();
    };

    let Expression::ArrowFunctionExpression(function) = &block.expression else {
        unreachable!();
    };

    function.scope_id.get().unwrap()
}

/// Check if a scope is within the Vue `<script setup>` block.
///
/// Uses scope ancestry to determine if the scope is a descendant
/// of the setup block's scope.
pub fn is_in_vue_setup(ctx: &LintContext<'_>, scope_id: ScopeId) -> bool {
    let setup_scope_id = get_vue_setup_scope_id(ctx);

    // Check if scope_id is setup_scope_id or a descendant
    let scopes = ctx.scoping();
    let mut current_scope = Some(scope_id);

    while let Some(current) = current_scope {
        if current == setup_scope_id {
            return true;
        }
        current_scope = scopes.scope_parent_id(current);
    }

    false
}

/// The last statement of `<script setup>` block must be JSXFragment, which includes Vue SFC struct
pub fn get_vue_sfc_struct<'a>(ctx: &LintContext<'a>) -> &'a Vec<'a, JSXChild<'a>> {
    let last_statement = get_vue_setup_statements(ctx).last().unwrap();
    let Statement::ExpressionStatement(expression_statement) = last_statement else {
        unreachable!();
    };
    let Expression::JSXFragment(jsx_fragment) = &expression_statement.expression else {
        unreachable!();
    };
    &jsx_fragment.children
}

// The start from the first statement, the end from the second to last statement
pub fn get_script_statements_span(ctx: &LintContext) -> Option<Span> {
    let statements = &ctx.nodes().program().body;
    if statements.len() > 1 {
        let first_statement = statements.first().unwrap();
        let second_to_last_statement = statements.get(statements.len() - 2).unwrap();
        Some(Span::new(first_statement.span().start, second_to_last_statement.span().end))
    } else {
        None
    }
}
