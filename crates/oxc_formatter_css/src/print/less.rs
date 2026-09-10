//! Less-specific printing: variable declarations, mixins, lookups, guards.

use oxc_css_parser::ast::{
    ComponentValue, LessCondition, LessConditionalQualifiedRule, LessConditions,
    LessDetachedRuleset, LessExtend, LessExtendList, LessExtendRule, LessMixinArgument,
    LessMixinCall, LessMixinDefinition, LessMixinName, LessNamespaceValue,
    LessNamespaceValueCallee, LessVariableDeclaration,
};
use oxc_formatter_core::{
    Buffer,
    builders::{
        group, hard_line_break, indent, soft_line_break_or_space, soft_line_indent_or_space, space,
        text,
    },
    write,
};

use crate::{
    comments::{BlockCommentAfter, FormatCommentBeforeContent},
    format::to_span,
    print::{
        CssFormatter, format_with, has_comments_between, is_glued,
        scss::write_top_level_list_element,
        selector,
        statement::{write_block, write_terminator_tail_comments, write_verbatim_prelude_rule},
        value::{self, ValueContext},
    },
};

/// `@name: value`. Returns `true` when the caller should append `;`.
pub(super) fn write_less_variable_declaration<'a>(
    decl: &LessVariableDeclaration<'a>,
    f: &mut CssFormatter<'_, 'a>,
) -> bool {
    let source = f.context().source_text();
    // Prettier's `shouldPrecededBySoftline` matches `css-decl` only,
    // never atrule-variables; see `ValueContext::no_leading_softline`.
    let value_ctx = ValueContext { no_leading_softline: true, ..ValueContext::default() };
    write!(f, "@");
    let name_span = to_span(decl.name.name.span());
    write!(f, text(source.text_for(&name_span)));
    let colon_end = to_span(&decl.colon_span).end;
    // Inline comments around the colon make postcss-less treat this as a plain at-rule:
    // the raw text is kept and the block loses its `;`.
    let value_start = to_span(decl.value.span()).start;
    let inline_before_colon = f.context().comments().iter_before(colon_end).any(|c| c.inline);
    let inline_after_colon = f
        .context()
        .comments()
        .iter_before(value_start)
        .any(|c| c.inline && c.span.start >= colon_end);
    let indented = if inline_before_colon {
        let _ = f.context().comments().take_before(value_start);
        let raw = source.slice_range(name_span.end, value_start);
        write!(f, text(raw.trim_end()));
        if let ComponentValue::LessDetachedRuleset(ruleset) = &decl.value {
            // Plain at-rule semantics: props are lowercased, no `;`
            if raw.trim_end().ends_with(':') {
                write!(f, space());
            } else {
                write!(f, hard_line_break());
            }
            write_block(&ruleset.block, f);
            return false;
        }
        write!(f, space());
        false
    } else if inline_after_colon {
        // Inline comment AFTER the colon only: still a variable;
        // the comment and line structure are kept (`@var: // c\n{`),
        // and a plain value continues one level under the name (Prettier's at-rule params indent).
        // A detached ruleset keeps its `{` under the name.
        write!(f, [":", space()]);
        !matches!(&decl.value, ComponentValue::LessDetachedRuleset(_))
    } else {
        let _ = f.context().comments().take_before(colon_end);
        write!(f, [":", space()]);
        false
    };
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        if inline_after_colon {
            for &comment in f.context().comments().take_before(value_start) {
                write!(f, FormatCommentBeforeContent::new(comment, BlockCommentAfter::HardLine));
            }
        }
        write_top_level_list_element(&decl.value, value_ctx, f);
    });
    if indented {
        // No `dedent` unlike `write_declaration`: the value opens no indent of its own (`no_leading_softline`)
        write!(f, indent(&body));
    } else {
        write!(f, body);
    }
    write_terminator_tail_comments(to_span(&decl.span).end, f);
    true
}

fn write_mixin_name<'a>(name: &LessMixinName<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let span = to_span(name.span());
    write!(f, text(source.text_for(&span)));
}

/// `.mixin(@params...) when (guard) { ... }`.
/// Prettier reads the prelude with postcss-selector-parser: the name and `when` are words joined by one space,
/// the parameter list is a token printed raw apart from number/string adjustments
/// (its spacing, `;` separators and line breaks survive), and any comment makes it verbatim.
/// `LessMixinParameters` is structured, so the parameters COULD break on width instead.
pub(super) fn write_less_mixin_definition<'a>(
    def: &LessMixinDefinition<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let start = to_span(def.name.span()).start;
    if has_comments_between(start, to_span(&def.block.span).start, f) {
        write_verbatim_prelude_rule(start, &def.block, true, f);
        return;
    }

    // The prelude breaks like a selector: at its word gaps, all at once, one indent in
    let prelude = format_with(|f: &mut CssFormatter<'_, 'a>| {
        write_mixin_name(&def.name, f);
        if !is_glued(def.name.span(), &def.params.span) {
            write!(f, soft_line_break_or_space());
        }
        let source = f.context().source_text();
        value::write_adjusted_verbatim(source.text_for(&to_span(&def.params.span)), f);
        if let Some(guard) = &def.guard {
            write_less_guard(guard, f);
        }
    });
    write!(f, [group(&indent(&prelude)), space()]);
    write_block(&def.block, f);
}

/// `selector when (guard) { ... }` — a `css-rule` in Prettier: the selector list,
/// the guard, the block, and NO trailing `;`.
/// Same verbatim bail-out on comments as a mixin definition.
pub(super) fn write_less_conditional_qualified_rule<'a>(
    rule: &LessConditionalQualifiedRule<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let start = to_span(&rule.span).start;
    if has_comments_between(start, to_span(&rule.block.span).start, f) {
        write_verbatim_prelude_rule(start, &rule.block, true, f);
        return;
    }

    let prelude = format_with(|f: &mut CssFormatter<'_, 'a>| {
        selector::write_selector_list(&rule.selector, selector::SelectorListStyle::Hard, f);
        write_less_guard(&rule.guard, f);
    });
    write!(f, [group(&indent(&prelude)), space()]);
    write_block(&rule.block, f);
}

/// ` when <cond>, <cond>`: each condition raw apart from number/string adjustments (Prettier's paren token),
/// the alternatives inline. Over the width the enclosing group breaks before `when` and after each `,`,
/// never inside `when <cond>` (see DIVERGENCES.md "less-guard-list-inline").
/// A `when(` glued to its condition stays glued (one selector word to Prettier).
fn write_less_guard<'a>(guard: &LessConditions<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    write!(f, [soft_line_break_or_space(), "when"]);
    for (i, condition) in guard.conditions.iter().enumerate() {
        if i > 0 {
            write!(f, [",", soft_line_break_or_space()]);
        } else if !is_glued(&guard.when_span, condition.span()) {
            write!(f, space());
        }
        value::write_adjusted_verbatim(source.text_for(&to_span(condition.span())), f);
    }
}

/// Statement-position `.mixin(args);` — a `mixin` at-rule in Prettier, whose
/// params are re-parsed as a SELECTOR (parser-postcss.js) and printed raw:
/// argument spacing is preserved and a long call never breaks on width.
///
/// NOTE: `oxc-css-parser` gives a structured `LessMixinCall` with callee + args,
/// so a structured printer (argument list, width-breaking) is possible.
/// We follow Prettier's verbatim contract so `.mixin(  @a , @b  )` etc, survive intact.
pub(super) fn write_less_mixin_call_statement<'a>(
    call: &LessMixinCall<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    let span = to_span(&call.span);
    let end = call.important.as_ref().map_or(span.end, |imp| to_span(&imp.span).start);
    let raw = source.slice_range(span.start, end).trim_end();
    let _ = f.context().comments().take_before(end);
    value::write_adjusted_verbatim(raw, f);
    if let Some(important) = &call.important {
        value::write_trailing_important(important, f);
    }
}

/// `.mixin(args) !important` in VALUE / namespace-callee position only
/// (statement position goes through `write_less_mixin_call_statement`).
fn write_less_mixin_call<'a>(call: &LessMixinCall<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    for child in &call.callee.children {
        if let Some(combinator) = &child.combinator {
            let span = to_span(&combinator.span);
            write!(f, [space(), text(source.text_for(&span)), space()]);
        }
        write_mixin_name(&child.name, f);
    }
    if let Some(args) = &call.args {
        write!(f, "(");
        let separator: &str = if args.is_comma_separated { ", " } else { "; " };
        for (i, arg) in args.args.iter().enumerate() {
            if i > 0 {
                write!(f, text(separator));
            }
            match arg {
                LessMixinArgument::Named(named) => {
                    let span = to_span(named.name.span());
                    write!(f, [text(source.text_for(&span)), ":", space()]);
                    value::write_component_value(&named.value, ValueContext::default(), f);
                }
                LessMixinArgument::Value(value) => {
                    value::write_component_value(value, ValueContext::default(), f);
                }
                LessMixinArgument::Variadic(variadic) => {
                    let span = to_span(variadic.name.span());
                    write!(f, [text(source.text_for(&span)), "..."]);
                }
            }
        }
        write!(f, ")");
    }
    if let Some(important) = &call.important {
        value::write_trailing_important(important, f);
    }
}

fn write_less_condition<'a>(condition: &LessCondition<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    match condition {
        LessCondition::Binary(binary) => {
            write_less_condition(&binary.left, f);
            let op_span = to_span(&binary.op.span);
            write!(f, [space(), text(source.text_for(&op_span)), space()]);
            write_less_condition(&binary.right, f);
        }
        LessCondition::Negated(negated) => {
            write!(f, ["not", space()]);
            write_less_condition(&negated.condition, f);
        }
        LessCondition::Parenthesized(paren) => {
            write!(f, "(");
            write_less_condition(&paren.condition, f);
            write!(f, ")");
        }
        LessCondition::Value(value) => {
            value::write_component_value(value, ValueContext::default(), f);
        }
    }
}

/// `.mixin(args)[@lookup][...]` / `@var[lookup]`
fn write_less_namespace_value<'a>(
    namespace: &LessNamespaceValue<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    match &namespace.callee {
        LessNamespaceValueCallee::LessMixinCall(call) => {
            write_less_mixin_call(call, f);
        }
        LessNamespaceValueCallee::LessVariable(variable) => {
            let span = to_span(variable.span());
            write!(f, text(source.text_for(&span)));
        }
    }
    for lookup in &namespace.lookups.lookups {
        write!(f, "[");
        if let Some(name) = &lookup.name {
            let span = to_span(name.span());
            write!(f, text(source.text_for(&span)));
        }
        write!(f, "]");
    }
}

/// `{ ... }` detached ruleset as a value.
/// Property names inside keep their case (Prettier checks the enclosing `variable` at-rule).
fn write_less_detached_ruleset<'a>(
    ruleset: &LessDetachedRuleset<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    // Block comments before `{` stay inline; `//` ends its line.
    let block_start = to_span(&ruleset.block.span).start;
    for &comment in f.context().comments().take_before(block_start) {
        write!(f, FormatCommentBeforeContent::new(comment, BlockCommentAfter::Space));
    }
    let was = f.context().in_less_detached().replace(true);
    write_block(&ruleset.block, f);
    f.context().in_less_detached().set(was);
}

/// Statement-position `&:extend(.a, .b)`.
/// Prints the SAME pseudo-args layout as the selector-position form
/// (inline when it fits; on overflow the parens take their own lines and the selectors break one per line).
/// NOTE: Prettier ALWAYS breaks multiple selectors.
pub(super) fn write_less_extend_rule<'a>(rule: &LessExtendRule<'a>, f: &mut CssFormatter<'_, 'a>) {
    write!(f, "&");
    if let Some(suffix) = &rule.nesting_selector.suffix {
        selector::write_interpolable_ident(suffix, f);
    }
    write!(f, ":");
    write!(f, text(rule.name_of_extend.raw));
    selector::write_pseudo_args_group(|f| write_less_extend_list(&rule.extend, f), f);
}

/// The `:extend(...)` argument list, shared by both positions:
/// selectors joined by `,` + breakable space.
pub(super) fn write_less_extend_list<'a>(list: &LessExtendList<'a>, f: &mut CssFormatter<'_, 'a>) {
    for (i, extend) in list.elements.iter().enumerate() {
        if i > 0 {
            write!(f, ",");
            write!(f, soft_line_break_or_space());
        }
        write_less_extend(extend, f);
    }
}

fn write_less_extend<'a>(extend: &LessExtend<'a>, f: &mut CssFormatter<'_, 'a>) {
    let Some(all) = &extend.all else {
        selector::write_complex_selector(&extend.selector, f);
        return;
    };
    // `all` acts as one more descendant term of the selector
    // (Prettier's selector parser absorbs it into the selector-root):
    // it breaks together with the selector, at the same +2.
    let body = format_with(|f: &mut CssFormatter<'_, 'a>| {
        selector::write_complex_selector(&extend.selector, f);
        write!(f, soft_line_indent_or_space(&text(all.raw)));
    });
    write!(f, group(&body));
}

/// `value` of a ComponentValue that is Less-specific; returns false if not handled.
pub(super) fn write_less_component_value<'a>(
    value: &ComponentValue<'a>,
    f: &mut CssFormatter<'_, 'a>,
) -> bool {
    match value {
        ComponentValue::LessMixinCall(call) => write_less_mixin_call(call, f),
        ComponentValue::LessNamespaceValue(namespace) => write_less_namespace_value(namespace, f),
        ComponentValue::LessDetachedRuleset(ruleset) => write_less_detached_ruleset(ruleset, f),
        ComponentValue::LessCondition(condition) => write_less_condition(condition, f),
        _ => return false,
    }
    true
}
