//! Less-specific printing: variable declarations, mixins, lookups, guards.

use oxc_formatter_core::{
    Buffer,
    builders::{space, text},
    write,
};
use raffia::{
    Spanned,
    ast::{
        ComponentValue, LessCondition, LessConditions, LessMixinArgument, LessMixinCall,
        LessMixinDefinition, LessMixinName, LessMixinParameter, LessVariableDeclaration,
    },
};

use crate::{
    format::to_span,
    print::{
        CssFormatter,
        statement::write_block,
        value::{self, ValueContext},
    },
};

/// `@name: value`. Returns `true` when the caller should append `;`.
pub fn write_less_variable_declaration<'a>(
    decl: &LessVariableDeclaration<'a>,
    f: &mut CssFormatter<'_, 'a>,
) -> bool {
    let source = f.context().source_text();
    write!(f, "@");
    let name_span = to_span(decl.name.name.span());
    write!(f, text(source.text_for(&name_span)));
    let colon_end = to_span(&decl.colon_span).end;
    // Inline comments around the colon make postcss-less treat this as a
    // plain at-rule: the raw text is kept and the block loses its `;`.
    let value_start_pos = to_span(decl.value.span()).start;
    let inline_before_colon = f.context().comments().iter_before(colon_end).any(|c| c.inline);
    let inline_after_colon = f
        .context()
        .comments()
        .iter_before(value_start_pos)
        .any(|c| c.inline && c.span.start >= colon_end);
    // Inline comment AFTER the colon only: still a variable; the comment and
    // line structure are kept (`@var: // c\n{`).
    if !inline_before_colon && inline_after_colon {
        write!(f, [":", space()]);
        for &comment in f.context().comments().take_before(value_start_pos) {
            crate::comments::write_single_comment(comment, f);
            write!(f, oxc_formatter_core::builders::hard_line_break());
        }
        crate::print::scss::write_top_level_value(&decl.value, ValueContext::default(), f);
        return true;
    }
    if inline_before_colon {
        let value_start = to_span(decl.value.span()).start;
        let _ = f.context().comments().take_before(value_start);
        let raw = source.slice_range(name_span.end, value_start);
        write!(f, text(raw.trim_end()));
        if let ComponentValue::LessDetachedRuleset(ruleset) = &decl.value {
            // Plain at-rule semantics: props are lowercased, no `;`.
            if raw.trim_end().ends_with(':') {
                write!(f, space());
            } else {
                write!(f, oxc_formatter_core::builders::hard_line_break());
            }
            write_block(&ruleset.block, f);
            return false;
        }
        write!(f, space());
        crate::print::scss::write_top_level_value(&decl.value, ValueContext::default(), f);
        return true;
    }
    // postcss-less drops (block) comments between the name and the colon.
    let _ = f.context().comments().take_before(colon_end);
    write!(f, [":", space()]);
    crate::print::scss::write_top_level_value(&decl.value, ValueContext::default(), f);
    true
}

fn write_mixin_name<'a>(name: &LessMixinName<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let span = to_span(name.span());
    write!(f, text(source.text_for(&span)));
}

/// `.mixin(@params...) when (guard) { ... }`
pub fn write_less_mixin_definition<'a>(
    def: &LessMixinDefinition<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    write_mixin_name(&def.name, f);
    write_mixin_parameters(&def.params, f);
    if let Some(guard) = &def.guard {
        write_less_guard(guard, f);
    }
    write!(f, space());
    write_block(&def.block, f);
}

fn write_mixin_parameters<'a>(
    params: &raffia::ast::LessMixinParameters<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    write!(f, "(");
    let separator: &str = if params.is_comma_separated { ", " } else { "; " };
    for (i, param) in params.params.iter().enumerate() {
        if i > 0 {
            write!(f, text(separator));
        }
        match param {
            LessMixinParameter::Named(named) => {
                let span = to_span(named.name.span());
                write!(f, text(source.text_for(&span)));
                if let Some(default) = &named.value {
                    write!(f, [":", space()]);
                    value::write_component_value(&default.value, ValueContext::default(), f);
                }
            }
            LessMixinParameter::Unnamed(unnamed) => {
                value::write_component_value(&unnamed.value, ValueContext::default(), f);
            }
            LessMixinParameter::Variadic(variadic) => {
                if let Some(name) = &variadic.name {
                    let span = to_span(name.span());
                    write!(f, text(source.text_for(&span)));
                }
                write!(f, "...");
            }
        }
    }
    write!(f, ")");
}

/// `.mixin(args) !important` — used both as a statement and as a value.
pub fn write_less_mixin_call<'a>(call: &LessMixinCall<'a>, f: &mut CssFormatter<'_, 'a>) {
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

/// ` when (cond), (cond) and (cond)`
pub fn write_less_guard<'a>(guard: &LessConditions<'a>, f: &mut CssFormatter<'_, 'a>) {
    write!(f, [space(), "when", space()]);
    for (i, condition) in guard.conditions.iter().enumerate() {
        if i > 0 {
            write!(f, [",", space()]);
        }
        write_less_condition(condition, f);
    }
}

pub fn write_less_condition<'a>(condition: &LessCondition<'a>, f: &mut CssFormatter<'_, 'a>) {
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
pub fn write_less_namespace_value<'a>(
    namespace: &raffia::ast::LessNamespaceValue<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    match &namespace.callee {
        raffia::ast::LessNamespaceValueCallee::LessMixinCall(call) => {
            write_less_mixin_call(call, f);
        }
        raffia::ast::LessNamespaceValueCallee::LessVariable(variable) => {
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

/// `{ ... }` detached ruleset as a value. Property names inside keep their
/// case (Prettier checks the enclosing `variable` at-rule).
pub fn write_less_detached_ruleset<'a>(
    ruleset: &raffia::ast::LessDetachedRuleset<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    // Comments before `{` stay on the same line.
    let block_start = to_span(&ruleset.block.span).start;
    for &comment in f.context().comments().take_before(block_start) {
        crate::comments::write_single_comment(comment, f);
        write!(f, space());
    }
    let was = f.context().in_less_detached().replace(true);
    write_block(&ruleset.block, f);
    f.context().in_less_detached().set(was);
}

/// `value` of a ComponentValue that is Less-specific; returns false if not handled.
pub fn write_less_component_value<'a>(
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
