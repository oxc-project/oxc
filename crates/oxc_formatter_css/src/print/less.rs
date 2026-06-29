//! Less-specific printing: variable declarations, mixins, lookups, guards.

use oxc_css_parser::{
    Spanned,
    ast::{
        ComponentValue, LessCondition, LessConditionalQualifiedRule, LessDetachedRuleset,
        LessMixinArgument, LessMixinCall, LessMixinDefinition, LessMixinName, LessNamespaceValue,
        LessNamespaceValueCallee, LessVariableDeclaration, SimpleBlock,
    },
};

use oxc_formatter_core::{
    Buffer, arena_cow_str,
    builders::{hard_line_break, space, text},
    write,
};

use crate::{
    comments::{last_line_has_inline_comment, write_single_comment},
    format::to_span,
    print::{
        CssFormatter,
        statement::write_block,
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
    let value_start_pos = to_span(decl.value.span()).start;
    let inline_before_colon = f.context().comments().iter_before(colon_end).any(|c| c.inline);
    let inline_after_colon = f
        .context()
        .comments()
        .iter_before(value_start_pos)
        .any(|c| c.inline && c.span.start >= colon_end);
    // Inline comment AFTER the colon only: still a variable;
    // the comment and line structure are kept (`@var: // c\n{`).
    if !inline_before_colon && inline_after_colon {
        write!(f, [":", space()]);
        for &comment in f.context().comments().take_before(value_start_pos) {
            write_single_comment(comment, f);
            write!(f, hard_line_break());
        }
        crate::print::scss::write_top_level_value(&decl.value, value_ctx, f);
        return true;
    }
    if inline_before_colon {
        let value_start = to_span(decl.value.span()).start;
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
        crate::print::scss::write_top_level_value(&decl.value, value_ctx, f);
        return true;
    }
    // postcss-less drops (block) comments between the name and the colon
    let _ = f.context().comments().take_before(colon_end);
    write!(f, [":", space()]);
    crate::print::scss::write_top_level_value(&decl.value, value_ctx, f);
    true
}

fn write_mixin_name<'a>(name: &LessMixinName<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let span = to_span(name.span());
    write!(f, text(source.text_for(&span)));
}

/// Raw source text with Prettier's string-level normalizations applied
/// (`adjustNumbers(adjustStrings(...))`).
/// The selector-side print path for everything postcss-selector-parser receives:
/// spacing and newlines stay verbatim and nothing ever breaks on line width.
fn write_adjusted_verbatim<'a>(raw: &'a str, f: &mut CssFormatter<'_, 'a>) {
    let adjusted = value::adjust_numbers_and_strings(raw, f.options());
    write!(f, text(arena_cow_str(&adjusted, f)));
}

/// Prelude printed verbatim from `start` to the block, then the block.
/// A trailing `//` comment pushes `{` to the next line
/// (selector-unknown's `lastLineHasInlineComment`).
fn write_verbatim_prelude_rule<'a>(
    start: u32,
    block: &SimpleBlock<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    let block_start = to_span(&block.span).start;
    let raw = source.slice_range(start, block_start).trim_end();
    let _ = f.context().comments().take_before(block_start);
    write_adjusted_verbatim(raw, f);
    if last_line_has_inline_comment(raw) {
        write!(f, hard_line_break());
    } else {
        write!(f, space());
    }
    write_block(block, f);
}

/// `.mixin(@params...) when (guard) { ... }`:
/// Prettier hands the whole prelude to postcss-selector-parser (`css-rule` selector)
/// and prints it raw apart from number/string adjustments,
/// so parameter spacing, a space before `(`, trailing `;` separators and multi-line layouts all survive.
///
/// NOTE: `oxc-css-parser` gives us a structured `LessMixinDefinition` (name + params + guard),
/// so we COULD print this structurally and break long parameter lists on width.
pub(super) fn write_less_mixin_definition<'a>(
    def: &LessMixinDefinition<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    write_verbatim_prelude_rule(to_span(def.name.span()).start, &def.block, f);
}

/// `selector when (guard) { ... }` — a `css-rule` in Prettier: raw selector
/// text (guard included), block, and NO trailing `;`.
///
/// NOTE: `oxc-css-parser` structures the selector and the `when` guard,
/// but we keep the raw source for Prettier alignment.
pub(super) fn write_less_conditional_qualified_rule<'a>(
    rule: &LessConditionalQualifiedRule<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    write_verbatim_prelude_rule(to_span(&rule.span).start, &rule.block, f);
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
    write_adjusted_verbatim(raw, f);
    if call.important.is_some() {
        write!(f, [space(), "!important"]);
    }
}

/// `.mixin(args) !important` in VALUE / namespace-callee position only
/// (statement position goes through `write_less_mixin_call_statement`).
fn write_less_mixin_call<'a>(call: &LessMixinCall<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    for child in &call.callee.children {
        if let Some(combinator) = &child.combinator {
            let span = to_span(combinator.span());
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
    if call.important.is_some() {
        write!(f, [space(), "!important"]);
    }
}

fn write_less_condition<'a>(condition: &LessCondition<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    match condition {
        LessCondition::Binary(binary) => {
            write_less_condition(&binary.left, f);
            let op_span = to_span(binary.op.span());
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
    // Comments before `{` stay on the same line
    let block_start = to_span(&ruleset.block.span).start;
    for &comment in f.context().comments().take_before(block_start) {
        write_single_comment(comment, f);
        write!(f, space());
    }
    let was = f.context().in_less_detached().replace(true);
    write_block(&ruleset.block, f);
    f.context().in_less_detached().set(was);
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
