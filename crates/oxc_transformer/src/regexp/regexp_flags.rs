use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{Atom, Span};

use std::rc::Rc;

use crate::TransformTarget;

/// Regexp
pub struct RegexpFlags<'a> {
    ast: Rc<AstBuilder<'a>>,
    transform_flags: RegExpFlags,
}

impl<'a> RegexpFlags<'a> {
    pub fn new_with_transform_target(
        ast: Rc<AstBuilder<'a>>,
        transform_target: TransformTarget,
    ) -> Option<Self> {
        let transform_flags = from_transform_target(transform_target);

        if transform_flags.is_empty() {
            None
        } else {
            Some(Self { ast, transform_flags })
        }
    }

    pub fn transform_expression<'b>(&mut self, expr: &'b mut Expression<'a>) {
        let Expression::RegExpLiteral(reg_literal) = expr else { return };

        if reg_literal.regex.flags.intersection(self.transform_flags).is_empty() {
            return;
        }

        let ident = IdentifierReference::new(Span::default(), Atom::from("RegExp"));
        let callee = self.ast.identifier_expression(ident);
        let pattern_literal = self
            .ast
            .string_literal(Span::default(), Atom::from(reg_literal.regex.pattern.as_str()));
        let flags_literal = self
            .ast
            .string_literal(Span::default(), Atom::from(reg_literal.regex.flags.to_string()));
        let pattern_literal = self.ast.literal_string_expression(pattern_literal);
        let flags_literal = self.ast.literal_string_expression(flags_literal);

        let mut arguments = self.ast.new_vec_with_capacity(2);
        arguments.push(Argument::Expression(pattern_literal));
        arguments.push(Argument::Expression(flags_literal));

        *expr = self.ast.new_expression(Span::default(), callee, arguments, None);
    }
}

fn from_transform_target(value: TransformTarget) -> RegExpFlags {
    let mut flag = RegExpFlags::empty();

    if value < TransformTarget::ESNext {
        flag |= RegExpFlags::I;
        flag |= RegExpFlags::M;
    }

    if value < TransformTarget::ES2024 {
        flag |= RegExpFlags::V;
    }

    if value < TransformTarget::ES2022 {
        flag |= RegExpFlags::D;
    }

    if value < TransformTarget::ES2018 {
        flag |= RegExpFlags::S;
    }

    if value < TransformTarget::ES2015 {
        flag |= RegExpFlags::Y;
        flag |= RegExpFlags::U;
    }

    flag
}
