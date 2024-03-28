use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{Atom, SPAN};

use std::rc::Rc;

use crate::{context::TransformerCtx, TransformOptions, TransformTarget};

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
    pub fn new(ctx: TransformerCtx<'a>) -> Option<Self> {
        let transform_flags = Self::from_transform_target(&ctx.options);
        (!transform_flags.is_empty()).then_some(Self { ast: ctx.ast, transform_flags })
    }

    fn from_transform_target(options: &TransformOptions) -> RegExpFlags {
        let target = options.target;
        let mut flag = RegExpFlags::empty();
        if target < TransformTarget::ES2015 || options.sticky_regex {
            flag |= RegExpFlags::Y;
        }
        if target < TransformTarget::ES2015 {
            flag |= RegExpFlags::U;
        }
        if target < TransformTarget::ES2018 {
            flag |= RegExpFlags::S;
        }
        if target < TransformTarget::ES2022 {
            flag |= RegExpFlags::D;
        }
        if target < TransformTarget::ES2024 {
            flag |= RegExpFlags::V;
        }
        if target < TransformTarget::ESNext {
            flag |= RegExpFlags::I;
            flag |= RegExpFlags::M;
        }
        flag
    }

    // `/regex/flags` -> `new RegExp('regex', 'flags')`
    pub fn transform_expression(&self, expr: &mut Expression<'a>) {
        let Expression::RegExpLiteral(literal) = expr else { return };
        let regex = literal.regex.clone();
        if regex.flags.intersection(self.transform_flags).is_empty() {
            return;
        }
        let ident = IdentifierReference::new(SPAN, Atom::from("RegExp"));
        let callee = self.ast.identifier_reference_expression(ident);
        let pattern = StringLiteral::new(SPAN, regex.pattern.clone());
        let flags = StringLiteral::new(SPAN, self.ast.new_atom(&regex.flags.to_string()));
        let pattern_literal = self.ast.literal_string_expression(pattern);
        let flags_literal = self.ast.literal_string_expression(flags);
        let mut arguments = self.ast.new_vec_with_capacity(2);
        arguments.push(Argument::Expression(pattern_literal));
        arguments.push(Argument::Expression(flags_literal));
        *expr = self.ast.new_expression(SPAN, callee, arguments, None);
    }
}
