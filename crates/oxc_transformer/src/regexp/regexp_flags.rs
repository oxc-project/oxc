use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{Atom, Span};

use std::rc::Rc;

use crate::TransformTarget;

/// Transforms unsupported regex flags into Regex constructors.
///
/// i.e. `/regex/flags` -> `new RegExp('regex', 'flags')`
///
/// * ES2024 [Unicode Sets v](https://babel.dev/docs/babel-plugin-transform-unicode-sets-regex)
/// * ES2022 [Match Indices d](https://github.com/tc39/proposal-regexp-match-indices)
/// * ES2018 [Dotall s](https://babel.dev/docs/babel-plugin-transform-dotall-regex)
/// * ES2015 [Unicode u](https://babel.dev/docs/babel-plugin-transform-unicode-regex)
/// * ES2015 [Sticky y](https://babel.dev/docs/babel-plugin-transform-sticky-regex)
pub struct RegexpFlags<'a> {
    ast: Rc<AstBuilder<'a>>,
    transform_flags: RegExpFlags,
}

impl<'a> RegexpFlags<'a> {
    pub fn new_with_transform_target(
        ast: Rc<AstBuilder<'a>>,
        transform_target: TransformTarget,
    ) -> Option<Self> {
        let transform_flags = Self::from_transform_target(transform_target);
        (!transform_flags.is_empty()).then(|| Self { ast, transform_flags })
    }

    // `/regex/flags` -> `new RegExp('regex', 'flags')`
    pub fn transform_expression(&self, expr: &mut Expression<'a>) {
        let Expression::RegExpLiteral(literal) = expr else { return };
        let regex = &literal.regex;
        if regex.flags.intersection(self.transform_flags).is_empty() {
            return;
        }
        let ident = IdentifierReference::new(Span::default(), Atom::from("RegExp"));
        let callee = self.ast.identifier_reference_expression(ident);
        let pattern = StringLiteral::new(Span::default(), Atom::from(regex.pattern.as_str()));
        let flags = StringLiteral::new(Span::default(), Atom::from(regex.flags.to_string()));
        let pattern_literal = self.ast.literal_string_expression(pattern);
        let flags_literal = self.ast.literal_string_expression(flags);
        let mut arguments = self.ast.new_vec_with_capacity(2);
        arguments.push(Argument::Expression(pattern_literal));
        arguments.push(Argument::Expression(flags_literal));
        *expr = self.ast.new_expression(Span::default(), callee, arguments, None);
    }

    fn from_transform_target(value: TransformTarget) -> RegExpFlags {
        let mut flag = RegExpFlags::empty();
        if value < TransformTarget::ES2015 {
            flag |= RegExpFlags::Y;
            flag |= RegExpFlags::U;
        }
        if value < TransformTarget::ES2018 {
            flag |= RegExpFlags::S;
        }
        if value < TransformTarget::ES2022 {
            flag |= RegExpFlags::D;
        }
        if value < TransformTarget::ES2024 {
            flag |= RegExpFlags::V;
        }
        if value < TransformTarget::ESNext {
            flag |= RegExpFlags::I;
            flag |= RegExpFlags::M;
        }
        flag
    }
}
