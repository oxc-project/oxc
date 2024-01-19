use oxc_ast::ast::Expression;

pub fn check_constructor<'a>(callee: &Expression<'a>, name: &str) -> bool {
    let Expression::Identifier(ident) = callee else {
        return false;
    };
    ident.name == name
}
