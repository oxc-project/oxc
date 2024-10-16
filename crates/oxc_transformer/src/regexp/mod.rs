//! RegExp Transformer
//!
//! This module supports various RegExp plugins to handle unsupported RegExp literal features.
//! When an unsupported feature is detected, these plugins convert the RegExp literal into
//! a `new RegExp()` constructor call to avoid syntax errors.
//!
//! Note: You will need to include a polyfill for the `RegExp` constructor in your code to have the correct runtime behavior.
//!
//! ### ES2015
//!
//! #### Sticky flag (`y`)
//! - @babel/plugin-transform-sticky-regex: <https://babeljs.io/docs/en/babel-plugin-transform-sticky-regex>
//!
//! #### Unicode flag (`u`)
//! - @babel/plugin-transform-unicode-regex: <https://babeljs.io/docs/en/babel-plugin-transform-unicode-regex>
//!
//! ### ES2018
//!
//! #### DotAll flag (`s`)
//! - @babel/plugin-transform-dotall-regex: <https://babeljs.io/docs/en/babel-plugin-transform-dotall-regex>
//! - Spec: ECMAScript 2018: <https://262.ecma-international.org/9.0/#sec-get-regexp.prototype.dotAll>
//!
//! #### Lookbehind assertions (`/(?<=x)/` and `/(?<!x)/`)
//! - Implementation: Same as esbuild's handling
//!
//! #### Named capture groups (`(?<name>x)`)
//! - @babel/plugin-transform-named-capturing-groups-regex: <https://babeljs.io/docs/en/babel-plugin-transform-named-capturing-groups-regex>
//!
//! #### Unicode property escapes (`\p{...}` and `\P{...}`)
//! - @babel/plugin-transform-unicode-property-regex: <https://babeljs.io/docs/en/babel-plugin-proposal-unicode-property-regex>
//!
//! ### ES2022
//!
//! #### Match indices flag (`d`)
//! - Implementation: Same as esbuild's handling
//!
//! ### ES2024
//!
//! #### Set notation + properties of strings (`v`)
//! - @babel/plugin-transform-unicode-sets-regex: <https://babeljs.io/docs/en/babel-plugin-proposal-unicode-sets-regex>
//! - TC39 Proposal: <https://github.com/tc39/proposal-regexp-set-notation>
//!
//! TODO(improve-on-babel): We could convert to plain `RegExp(...)` instead of `new RegExp(...)`.
//! TODO(improve-on-babel): When flags is empty, we could output `RegExp("(?<=x)")` instead of `RegExp("(?<=x)", "")`.
//! (actually these would be improvements on ESBuild, not Babel)

use std::borrow::Cow;

use oxc_ast::{ast::*, NONE};
use oxc_diagnostics::Result;
use oxc_regular_expression::ast::{
    CharacterClass, CharacterClassContents, LookAroundAssertionKind, Pattern, Term,
};
use oxc_semantic::ReferenceFlags;
use oxc_span::{Atom, SPAN};
use oxc_traverse::{Traverse, TraverseCtx};

use crate::TransformCtx;

mod options;

pub use options::RegExpOptions;

pub struct RegExp<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
    unsupported_flags: RegExpFlags,
    some_unsupported_patterns: bool,
    look_behind_assertions: bool,
    named_capture_groups: bool,
    unicode_property_escapes: bool,
}

impl<'a, 'ctx> RegExp<'a, 'ctx> {
    pub fn new(options: RegExpOptions, ctx: &'ctx TransformCtx<'a>) -> Self {
        // Get unsupported flags
        let mut unsupported_flags = RegExpFlags::empty();
        if options.dot_all_flag {
            unsupported_flags |= RegExpFlags::S;
        }
        if options.sticky_flag {
            unsupported_flags |= RegExpFlags::Y;
        }
        if options.unicode_flag {
            unsupported_flags |= RegExpFlags::U;
        }
        if options.match_indices {
            unsupported_flags |= RegExpFlags::D;
        }
        if options.set_notation {
            unsupported_flags |= RegExpFlags::V;
        }

        // Get if some unsupported patterns
        let RegExpOptions {
            look_behind_assertions,
            named_capture_groups,
            unicode_property_escapes,
            ..
        } = options;

        let some_unsupported_patterns =
            look_behind_assertions || named_capture_groups || unicode_property_escapes;

        Self {
            ctx,
            unsupported_flags,
            some_unsupported_patterns,
            look_behind_assertions,
            named_capture_groups,
            unicode_property_escapes,
        }
    }
}

impl<'a, 'ctx> Traverse<'a> for RegExp<'a, 'ctx> {
    fn enter_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut oxc_traverse::TraverseCtx<'a>,
    ) {
        let Expression::RegExpLiteral(regexp) = expr else {
            return;
        };
        let regexp = regexp.as_mut();

        let flags = regexp.regex.flags;
        let has_unsupported_flags = flags.intersects(self.unsupported_flags);
        if !has_unsupported_flags {
            if !self.some_unsupported_patterns {
                // This RegExp has no unsupported flags, and there are no patterns which may need transforming,
                // so there's nothing to do
                return;
            }

            let span = regexp.span;
            let pattern = match &mut regexp.regex.pattern {
                RegExpPattern::Raw(raw) => {
                    // Try to parse pattern
                    match try_parse_pattern(raw, span, flags, ctx) {
                        Ok(pattern) => {
                            regexp.regex.pattern = RegExpPattern::Pattern(ctx.alloc(pattern));
                            let RegExpPattern::Pattern(pattern) = &regexp.regex.pattern else {
                                unreachable!()
                            };
                            pattern
                        }
                        Err(error) => {
                            regexp.regex.pattern = RegExpPattern::Invalid(raw);
                            self.ctx.error(error);
                            return;
                        }
                    }
                }
                RegExpPattern::Invalid(_) => return,
                RegExpPattern::Pattern(pattern) => &**pattern,
            };

            if !self.has_unsupported_regular_expression_pattern(pattern) {
                return;
            }
        }

        let pattern_source: Cow<'_, str> = match &regexp.regex.pattern {
            RegExpPattern::Raw(raw) => Cow::Borrowed(raw),
            RegExpPattern::Pattern(p) => Cow::Owned(p.to_string()),
            RegExpPattern::Invalid(_) => return,
        };

        let callee = {
            let symbol_id = ctx.scopes().find_binding(ctx.current_scope_id(), "RegExp");
            let ident = ctx.create_reference_id(
                SPAN,
                Atom::from("RegExp"),
                symbol_id,
                ReferenceFlags::read(),
            );
            ctx.ast.expression_from_identifier_reference(ident)
        };

        let mut arguments = ctx.ast.vec_with_capacity(2);
        arguments.push(
            ctx.ast.argument_expression(ctx.ast.expression_string_literal(SPAN, pattern_source)),
        );

        let flags_str = flags.to_string();
        let flags_str =
            ctx.ast.argument_expression(ctx.ast.expression_string_literal(SPAN, flags_str));
        arguments.push(flags_str);

        *expr = ctx.ast.expression_new(regexp.span, callee, arguments, NONE);
    }
}

impl<'a, 'ctx> RegExp<'a, 'ctx> {
    /// Check if the regular expression contains any unsupported syntax.
    ///
    /// Based on parsed regular expression pattern.
    fn has_unsupported_regular_expression_pattern(&self, pattern: &Pattern<'a>) -> bool {
        pattern.body.body.iter().any(|alternative| {
            alternative.body.iter().any(|term| self.term_contains_unsupported(term))
        })
    }

    fn term_contains_unsupported(&self, mut term: &Term) -> bool {
        // Loop because `Term::Quantifier` contains a nested `Term`
        loop {
            match term {
                Term::CapturingGroup(_) => return self.named_capture_groups,
                Term::UnicodePropertyEscape(_) => return self.unicode_property_escapes,
                Term::CharacterClass(character_class) => {
                    return self.unicode_property_escapes
                        && character_class_has_unicode_property_escape(character_class)
                }
                Term::LookAroundAssertion(assertion) => {
                    return self.look_behind_assertions
                        && matches!(
                            assertion.kind,
                            LookAroundAssertionKind::Lookbehind
                                | LookAroundAssertionKind::NegativeLookbehind
                        )
                }
                Term::Quantifier(quantifier) => term = &quantifier.body,
                _ => return false,
            }
        }
    }
}

fn character_class_has_unicode_property_escape(character_class: &CharacterClass) -> bool {
    character_class.body.iter().any(|element| match element {
        CharacterClassContents::UnicodePropertyEscape(_) => true,
        CharacterClassContents::NestedCharacterClass(character_class) => {
            character_class_has_unicode_property_escape(character_class)
        }
        _ => false,
    })
}

fn try_parse_pattern<'a>(
    raw: &'a str,
    span: Span,
    flags: RegExpFlags,
    ctx: &mut TraverseCtx<'a>,
) -> Result<Pattern<'a>> {
    use oxc_regular_expression::{Parser, ParserOptions};

    let options = ParserOptions::default()
        .with_span_offset(span.start + 1) // exclude `/`
        .with_flags(&flags.to_string());
    Parser::new(ctx.ast.allocator, raw, options).parse()
}
