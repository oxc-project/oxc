//! Contextual name inference shared by class transforms.

use oxc_ast::ast::AssignmentTarget;
use oxc_str::Str;
use oxc_traverse::Ancestor;

/// Infer an anonymous class's name from its immediate non-parenthesized ancestor.
///
/// Property names captured during class or object evaluation are handled by the caller.
pub fn contextual_class_name<'a>(ancestor: &Ancestor<'a, '_>) -> Option<Str<'a>> {
    match ancestor {
        Ancestor::VariableDeclaratorInit(decl) => decl.id().get_identifier_name().map(Str::from),
        Ancestor::AssignmentPatternRight(pattern) => {
            pattern.left().get_identifier_name().map(Str::from)
        }
        Ancestor::FormalParameterInitializer(param) => {
            param.pattern().get_identifier_name().map(Str::from)
        }
        Ancestor::AssignmentExpressionRight(assign)
            if assign.operator().is_assign() || assign.operator().is_logical() =>
        {
            assignment_name(assign.left())
        }
        Ancestor::AssignmentTargetWithDefaultInit(target) => assignment_name(target.binding()),
        Ancestor::AssignmentTargetPropertyIdentifierInit(target) => {
            Some(Str::from(target.binding().name))
        }
        Ancestor::ExportDefaultDeclarationDeclaration(_) => Some(Str::from("default")),
        _ => None,
    }
}

fn assignment_name<'a>(target: &AssignmentTarget<'a>) -> Option<Str<'a>> {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(ident) => Some(ident.name.into()),
        _ => None,
    }
}
