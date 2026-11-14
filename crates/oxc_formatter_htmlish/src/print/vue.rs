//! Vue: the JS of `{{ }}` and directive values, formatted by the session's dispatcher as fragments
//! in the file's script language. A part the dispatcher does not format prints as written.

use std::borrow::Cow;

use oxc_formatter_core::{
    builders::{
        exact_line_breaks, expand_parent, soft_line_indent_or_space, soft_space_or_block_indent,
        space, text,
    },
    dispatch_fragment_ir, format_args, write,
};
use oxc_htmlish_parser::{
    Span,
    ast::{Attribute, AttributeKind, NodeId, VueFor},
};

use super::{
    HtmlishFormatter,
    attribute::unescaped,
    embed::{self, dispatch_attribute_value, write_quoted},
    format_ir,
    tag::closing_end_borrowed,
};
use crate::tree::Tree;

/// Vue's component prop key: camelize `/-\w/g`, where `\w` is ASCII.
/// See [Vue's `camelize`](https://github.com/vuejs/core/blob/v3.5.45/packages/shared/src/general.ts).
pub(super) fn prop_name(name: &str) -> Cow<'_, str> {
    let Some(first_dash) = name.find('-') else { return Cow::Borrowed(name) };
    let mut result = String::with_capacity(name.len());
    result.push_str(&name[..first_dash]);
    let mut chars = name[first_dash..].chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '-'
            && chars.peek().is_some_and(|next| next.is_ascii_alphanumeric() || *next == '_')
        {
            result.push(chars.next().unwrap().to_ascii_uppercase());
        } else {
            result.push(ch);
        }
    }
    Cow::Owned(result)
}

/// Parent→child context for a JS fragment of a Vue template,
/// dispatched in the file's script language (`Tree::vue_script_language`).
///
/// The host translates it into JS's `JsRoot` and `JsEmbeddedIn`.
/// Every one but an interpolation is a `"`-quoted attribute value, whose strings print in `'`.
/// `tailwind_classes`: its strings are Tailwind classes, sorted as prettier-plugin-tailwindcss does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsInVueTemplate {
    /// `{{ }}`: an expression the interpolation lays out.
    Interpolation,
    /// `:a` / `.a` / `v-bind:a`: an expression; an object, an array, a string or a template literal hugs the quotes.
    Binding { tailwind_classes: bool },
    /// Another directive's value (`v-if`): an object or an array hugs the quotes.
    /// `in_parens`: Vue puts it in parens (`v-if`, `v-else-if`).
    Expression { tailwind_classes: bool, in_parens: bool },
    /// An event handler's value (`@a` / `v-on:a`) that is an expression, as `Expression` in parens (`$event => (…)`):
    /// Vue reads it as statements once it holds a `;`.
    HandlerExpression,
    /// An event handler's value that is no expression, but statements (`@a="b(); c()"`).
    Handler,
    /// `v-for`'s alias list (`(item, index)`), in parens when more than one.
    ForAlias,
    /// `v-for`'s source.
    ForSource,
    /// Slot props (`#a="{ b }"`, `v-slot`).
    SlotProps,
    /// A `<script>` block's `generic` (`T extends X`).
    ScriptGeneric,
}

/// How a Vue attribute's value is read, by its name.
#[derive(Clone, Copy)]
pub enum Value {
    Fragment(JsInVueTemplate),
    /// `v-for`'s `aliases in source`; `None` for a value Vue does not read so.
    For(Option<VueFor>),
    /// An event handler's: an expression, or statements.
    Handler,
}

/// `None` for an attribute whose value is no JS.
pub fn value_kind(
    t: &Tree<'_>,
    element: NodeId,
    attribute: &Attribute<'_>,
    f: &HtmlishFormatter<'_, '_>,
) -> Option<Value> {
    let source = t.source();
    let name = attribute.name.slice(source);
    let AttributeKind::Vue(directive) = &attribute.kind else {
        // `slot-scope`, `setup` and `vars` are plain attributes (apps/oxfmt/DIVERGENCES.md#js-in-vue-removed-attributes-as-written)
        return (name == "generic" && t.is_sfc_block(element) && t.tag(element) == "script")
            .then_some(Value::Fragment(JsInVueTemplate::ScriptGeneric));
    };
    // Tailwind classes, as prettier-plugin-tailwindcss reads its `tailwindAttributes` in Vue, by the attribute's whole name:
    // bound `class` and listed `a` / `:a` / `v-bind:a` (`:a`, `v-bind:a`) without modifiers, a listed directive as written
    let tailwind = f.options().sort_tailwindcss.as_deref();
    let listed = tailwind.unwrap_or_default();
    let bound_classes = || {
        tailwind.is_some()
            && directive.modifiers.is_empty()
            && directive.argument.filter(|argument| !argument.dynamic).is_some_and(|argument| {
                let argument = argument.span.slice(source);
                argument == "class"
                    || listed.iter().any(|listed| {
                        [
                            Some(listed.as_str()),
                            listed.strip_prefix(':'),
                            listed.strip_prefix("v-bind:"),
                        ]
                        .contains(&Some(argument))
                    })
            })
    };
    let has_argument = directive.argument.is_some();
    Some(match directive.name.slice(source) {
        "for" if name == "v-for" => Value::For(directive.for_expression.as_deref().copied()),
        "#" | "slot" => Value::Fragment(JsInVueTemplate::SlotProps),
        "@" => Value::Handler,
        "on" if has_argument => Value::Handler,
        ":" => Value::Fragment(JsInVueTemplate::Binding { tailwind_classes: bound_classes() }),
        "." => Value::Fragment(JsInVueTemplate::Binding { tailwind_classes: false }),
        "bind" if has_argument => {
            Value::Fragment(JsInVueTemplate::Binding { tailwind_classes: bound_classes() })
        }
        directive_name => Value::Fragment(JsInVueTemplate::Expression {
            tailwind_classes: listed.iter().any(|listed| listed == name),
            in_parens: matches!(directive_name, "if" | "else-if"),
        }),
    })
}

/// Writes a Vue attribute's (unescaped) `value` of `kind` formatted as JS, quoted, a `"` in it as `&quot;`;
/// `false` when the dispatcher leaves it as written,
/// as one ending in a line comment no line break closes (`apps/oxfmt/DIVERGENCES.md#js-in-vue-trailing-line-comment`).
pub fn write_attribute_value<'a>(
    t: &'a Tree<'a>,
    kind: Value,
    value: &'a str,
    f: &mut HtmlishFormatter<'_, 'a>,
) -> bool {
    let language = t.vue_script_language();
    match kind {
        Value::Fragment(fragment) => embed::write_attribute_value(language, &fragment, value, f),
        Value::Handler => {
            embed::write_attribute_value(language, &JsInVueTemplate::HandlerExpression, value, f)
                || embed::write_attribute_value(language, &JsInVueTemplate::Handler, value, f)
        }
        Value::For(v_for) => {
            let source = t.source();
            let Some(v_for) = v_for.filter(|v_for| formats_v_for(v_for, source)) else {
                return false;
            };
            let dispatch = |part: JsInVueTemplate, span: Span, f: &HtmlishFormatter<'_, 'a>| {
                dispatch_attribute_value(language, &part, unescaped(span.slice(source), f), f)
            };
            let Some(aliases) = dispatch(JsInVueTemplate::ForAlias, v_for.aliases, f) else {
                return false;
            };
            let Some(iterated) = dispatch(JsInVueTemplate::ForSource, v_for.source, f) else {
                return false;
            };
            let operator = text(v_for.operator.slice(source));
            let (aliases, iterated) = (format_ir(aliases), format_ir(iterated));
            write_quoted(&format_args!(aliases, space(), operator, space(), iterated), f);
            true
        }
    }
}

/// Writes `{{ }}`'s expression between its markers: formatted, or as written (as Prettier does).
pub fn write_interpolation<'a>(
    t: &Tree<'_>,
    id: NodeId,
    expression: &'a str,
    f: &mut HtmlishFormatter<'_, 'a>,
) {
    // The next sibling prints the `}}` right after this
    let borrowed = closing_end_borrowed(t, id);
    // The code Vue compiles, trimmed
    let code = expression.trim_ascii();
    if let Some(ir) = dispatch_fragment_ir(
        f,
        t.vue_script_language(),
        code,
        Some(&JsInVueTemplate::Interpolation),
    ) {
        if borrowed {
            write!(f, [soft_line_indent_or_space(&format_ir(ir)), space()]);
        } else {
            write!(f, soft_space_or_block_indent(&format_ir(ir)));
        }
        return;
    }
    // Whitespace only prints nothing (`{{}}`), as an Angular one does
    if code.is_empty() {
        return;
    }
    // A trailing line break prints at the indentation: the next line's `}}` lines up with the `{{`
    let Some(lf) =
        expression.rfind('\n').filter(|&lf| expression[lf + 1..].trim_ascii().is_empty())
    else {
        write!(f, text(expression));
        return;
    };
    let content = expression[..lf].trim_end_matches('\n');
    write!(f, text(content));
    // Its own line break before a borrowed `}}` is the one between the siblings
    let breaks = lf - content.len() + usize::from(!borrowed);
    if breaks > 0 {
        write!(f, exact_line_breaks(breaks));
    }
    if borrowed {
        write!(f, expand_parent());
    }
}

/// Whether `v-for`'s parts format: not when formatting would make Vue read them otherwise, which prints as written.
fn formats_v_for(v_for: &VueFor, source: &str) -> bool {
    // A spaced `in` / `of` in the aliases (`'k'in o` formats as `'k' in o`) would read as the operator.
    // One in a string or a comment counts too, for nothing; an AST check needs a `v-for` marker on `JsRoot::FunctionParams`
    let operator_in_aliases = v_for
        .aliases
        .slice(source)
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '$'))
        .any(|word| word == "in" || word == "of");
    // A name is missing before one that is not (`, i`)
    let name_missing = v_for.value.is_none() || (v_for.key.is_none() && v_for.index.is_some());
    !operator_in_aliases && !name_missing
}
