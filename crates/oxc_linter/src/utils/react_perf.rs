use oxc_ast::ast::Expression;

pub fn is_constructor_matching_name(callee: &Expression<'_>, name: &str) -> bool {
    let Expression::Identifier(ident) = callee else {
        return false;
    };
    ident.name == name
}
