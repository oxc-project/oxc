use crate::{ContextSubHost, LintContext};
use oxc_ast::{
    AstKind,
    ast::{
        CallExpression, ExportDefaultDeclarationKind, Expression, IdentifierReference,
        ObjectPropertyKind, Statement,
    },
};
use oxc_semantic::ScopeId;

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
pub fn has_default_exports_property(others: &Vec<&ContextSubHost<'_>>, check_name: &str) -> bool {
    for host in others {
        for other_node in host.semantic().nodes() {
            let AstKind::ExportDefaultDeclaration(export) = other_node.kind() else {
                continue;
            };

            let ExportDefaultDeclarationKind::ObjectExpression(export_obj) = &export.declaration
            else {
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
            if !is_non_local_reference(identifier, ctx) {
                return Some(DefineMacroProblem::ReferencingLocally);
            }
            None
        }
        _ => Some(DefineMacroProblem::EventsNotDefined),
    }
}

fn is_non_local_reference(identifier: &IdentifierReference, ctx: &LintContext<'_>) -> bool {
    if let Some(symbol_id) = ctx.semantic().scoping().get_root_binding(identifier.name) {
        return matches!(
            ctx.semantic().symbol_declaration(symbol_id).kind(),
            AstKind::ImportSpecifier(_)
        );
    }

    // variables outside the current `<script>` block are valid.
    // This is the same for unresolved variables.
    true
}

/// Get the scope ID of the Vue setup block (last top-level BlockStatement).
///
/// According to https://github.com/liangmiQwQ/vue-oxc-toolkit/blob/main/MAPPING.md,
/// In vue-oxc-toolkit compiled AST, the last top-level BlockStatement contains
/// the `<script setup>` code.
pub fn get_vue_setup_scope_id(ctx: &LintContext<'_>) -> ScopeId {
    let program = ctx.nodes().program();

    // Find the last top-level BlockStatement
    let Statement::BlockStatement(block) = program.body.last().unwrap() else {
        unreachable!();
    };

    block.scope_id.get().unwrap()
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
