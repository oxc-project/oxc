use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{Span, Atom};

use std::rc::Rc;

/// ES2015: Sticky Regex
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-sticky-regex>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-sticky-regex>
pub struct StickyRegex<'a> {
    ast: Rc<AstBuilder<'a>>,
}

impl<'a> StickyRegex<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>) -> Self {
        Self { ast }
    }

    pub fn transform_expression<'b>(&mut self, expr: &'b mut Expression<'a>) {
        let Expression::RegExpLiteral(reg_literal) = expr else { return };
        if !reg_literal.regex.flags.contains(RegExpFlags::Y) {
            return;
        }

        let ident = IdentifierReference::new(Span::default(), Atom::from("RegExp"));
        let callee = self.ast.identifier_expression(ident);
        let string_of_pattern = self.ast.string_literal(Span::default(), Atom::from(reg_literal.regex.pattern.as_str()));
        let string_of_flags = self.ast.string_literal(Span::default(), Atom::from(reg_literal.regex.flags.to_string()));
        let pattern_literal = self.ast.literal_string_expression(string_of_pattern);
        let flags_literal = self.ast.literal_string_expression(string_of_flags);

        let mut arguments = self.ast.new_vec_with_capacity(2);
        arguments.push(Argument::Expression(pattern_literal));
        arguments.push(Argument::Expression(flags_literal));

        *expr = self.ast.new_expression(Span::default(), callee, arguments, None);
    }
}
