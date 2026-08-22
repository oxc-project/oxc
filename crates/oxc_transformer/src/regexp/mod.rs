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
//! ### ES2025
//!
//! #### Duplicate named capture groups (`(?<name>x)|(?<name>y)`)
//! - @babel/plugin-transform-duplicate-named-capturing-groups-regex: <https://babeljs.io/docs/babel-plugin-transform-duplicate-named-capturing-groups-regex>
//!
//! TODO(improve-on-babel): We could convert to plain `RegExp(...)` instead of `new RegExp(...)`.
//! TODO(improve-on-babel): When flags is empty, we could output `RegExp("(?<=x)")` instead of `RegExp("(?<=x)", "")`.
//! (actually these would be improvements on ESBuild, not Babel)

use oxc_allocator::{ArenaVec, GetAllocator, TakeIn};
use oxc_ast::ast::*;
use oxc_regular_expression::{
    RegexUnsupportedPatterns, has_unsupported_regular_expression_pattern,
};
use oxc_semantic::ReferenceFlags;
use oxc_span::SPAN;
use oxc_str::static_ident;
use oxc_traverse::{Ancestor, Traverse};

use crate::{
    common::helper_loader::{Helper, helper_call_expr},
    context::TraverseCtx,
    state::TransformState,
};

mod duplicate_named_capture_groups;
mod options;

use duplicate_named_capture_groups::{NamedCaptureGroup, rewrite_duplicate_named_capture_groups};

pub use options::RegExpOptions;

pub struct RegExp {
    unsupported_flags: RegExpFlags,
    some_unsupported_patterns: bool,
    unsupported_patterns: RegexUnsupportedPatterns,
    duplicate_named_capture_groups_runtime: bool,
}

impl RegExp {
    pub fn new(options: RegExpOptions) -> Self {
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
            duplicate_named_capture_groups,
            duplicate_named_capture_groups_runtime,
            ..
        } = options;

        let some_unsupported_patterns = look_behind_assertions
            || named_capture_groups
            || unicode_property_escapes
            || duplicate_named_capture_groups;

        Self {
            unsupported_flags,
            some_unsupported_patterns,
            unsupported_patterns: RegexUnsupportedPatterns {
                look_behind_assertions,
                named_capture_groups,
                duplicate_named_capture_groups,
                unicode_property_escapes,
                pattern_modifiers: false,
            },
            duplicate_named_capture_groups_runtime,
        }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for RegExp {
    // `#[inline]` to avoid cost of function call for all `Expression`s which aren't `RegExpLiteral`s
    #[inline]
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if matches!(expr, Expression::RegExpLiteral(_)) {
            self.transform_regexp(expr, ctx);
        }
    }
}

impl<'a> RegExp {
    /// Lower unsupported duplicate named groups, then transform remaining unsupported syntax or
    /// flags to `new RegExp(...)`.
    fn transform_regexp(&self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let is_regexp_test = Self::is_regexp_test(ctx);
        let groups = self.rewrite_duplicate_named_capture_groups(expr, ctx);

        let Expression::RegExpLiteral(regexp) = expr else {
            unreachable!();
        };
        let regexp = regexp.as_mut();

        let pattern_text = regexp.regex.pattern.text;
        let flags = regexp.regex.flags;
        let has_unsupported_flags = flags.intersects(self.unsupported_flags);
        if !has_unsupported_flags {
            if !self.some_unsupported_patterns {
                // This RegExp has no unsupported flags, and there are no patterns which may need transforming,
                // so there's nothing to do
                return;
            }

            let owned_pattern;
            let pattern = if let Some(pattern) = &regexp.regex.pattern.pattern {
                pattern
            } else {
                match regexp.parse_pattern(ctx.allocator()) {
                    Ok(pattern) => {
                        owned_pattern = Some(pattern);
                        owned_pattern.as_ref().unwrap()
                    }
                    Err(error) => {
                        ctx.state.error(error);
                        return;
                    }
                }
            };

            if !has_unsupported_regular_expression_pattern(pattern, &self.unsupported_patterns) {
                if let Some(groups) = groups
                    && self.duplicate_named_capture_groups_runtime
                    && !is_regexp_test
                {
                    Self::wrap_regexp(expr, groups, ctx);
                }
                return;
            }
        }

        let callee = {
            let regexp = static_ident!("RegExp");
            let symbol_id = ctx.scoping().find_binding(ctx.current_scope_id(), regexp);
            ctx.create_ident_expr(SPAN, regexp, symbol_id, ReferenceFlags::read())
        };

        let arguments = [
            Argument::new_string_literal(SPAN, pattern_text, None, ctx),
            Argument::new_string_literal(
                SPAN,
                Str::from_str_in(flags.to_inline_string().as_str(), ctx),
                None,
                ctx,
            ),
        ];

        *expr = Expression::new_new_expression(regexp.span, callee, None, arguments, ctx);

        if let Some(groups) = groups
            && self.duplicate_named_capture_groups_runtime
            && !is_regexp_test
        {
            Self::wrap_regexp(expr, groups, ctx);
        }
    }

    fn rewrite_duplicate_named_capture_groups(
        &self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Vec<NamedCaptureGroup>> {
        if !self.unsupported_patterns.duplicate_named_capture_groups {
            return None;
        }

        let Expression::RegExpLiteral(regexp) = expr else {
            unreachable!();
        };
        let regexp = regexp.as_mut();

        let owned_pattern;
        let pattern = if let Some(pattern) = &regexp.regex.pattern.pattern {
            pattern
        } else {
            match regexp.parse_pattern(ctx.allocator()) {
                Ok(pattern) => {
                    owned_pattern = Some(pattern);
                    owned_pattern.as_ref().unwrap()
                }
                Err(error) => {
                    ctx.state.error(error);
                    return None;
                }
            }
        };

        let pattern_offset = regexp.span.start + 1;
        let result = rewrite_duplicate_named_capture_groups(
            regexp.regex.pattern.text.as_str(),
            pattern,
            pattern_offset,
        )?;

        regexp.regex.pattern.text = Str::from_str_in(&result.pattern, ctx);
        regexp.regex.pattern.pattern = None;
        regexp.raw = None;

        Some(result.groups)
    }

    fn wrap_regexp(
        expr: &mut Expression<'a>,
        groups: Vec<NamedCaptureGroup>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let properties = ArenaVec::from_iter_in(
            groups.into_iter().map(|group| {
                let name = Str::from_str_in(&group.name, ctx);
                let (key, computed) = if name == "__proto__" {
                    (PropertyKey::new_string_literal(SPAN, name, None, ctx), true)
                } else {
                    (PropertyKey::new_static_identifier(SPAN, name, ctx), false)
                };
                let value = if group.indices.len() == 1 {
                    Expression::new_numeric_literal(
                        SPAN,
                        f64::from(group.indices[0]),
                        None,
                        NumberBase::Decimal,
                        ctx,
                    )
                } else {
                    Expression::new_array_expression(
                        SPAN,
                        ArenaVec::from_iter_in(
                            group.indices.into_iter().map(|index| {
                                ArrayExpressionElement::new_numeric_literal(
                                    SPAN,
                                    f64::from(index),
                                    None,
                                    NumberBase::Decimal,
                                    ctx,
                                )
                            }),
                            ctx,
                        ),
                        ctx,
                    )
                };
                ObjectPropertyKind::new_object_property(
                    SPAN,
                    PropertyKind::Init,
                    key,
                    value,
                    false,
                    false,
                    computed,
                    ctx,
                )
            }),
            ctx,
        );
        let group_map = Expression::new_object_expression(SPAN, properties, ctx);
        let regexp = expr.take_in(ctx);
        let arguments =
            ArenaVec::from_array_in([Argument::from(regexp), Argument::from(group_map)], ctx);
        *expr = helper_call_expr(Helper::WrapRegExp, arguments, ctx);
    }

    fn is_regexp_test(ctx: &TraverseCtx<'a>) -> bool {
        matches!(
            ctx.parent(),
            Ancestor::StaticMemberExpressionObject(member) if member.property().name == "test"
        )
    }
}
