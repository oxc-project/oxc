//! Vue: the JS of `{{ }}` and directive values, formatted by the session's dispatcher as fragments
//! in the file's script language. A part the dispatcher does not format prints as written.

use oxc_formatter_core::{
    builders::{
        exact_line_breaks, expand_parent, soft_line_indent_or_space, soft_space_or_block_indent,
        space, text,
    },
    dispatch_fragment_ir, format_args, write,
};
use oxc_htmlish_parser::ast::{Attribute, AttributeKind, NodeId};

use super::{
    HtmlishFormatter,
    embed::{self, dispatch_attribute_value, write_quoted},
    format_ir,
    tag::closing_end_borrowed,
};
use crate::tree::Tree;

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
    /// `v-for`'s `alias in source`.
    For,
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
        "for" if name == "v-for" => Value::For,
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
        Value::For => {
            let Some(v_for) = split_v_for(value) else { return false };
            let dispatch = |part: JsInVueTemplate, text, f: &HtmlishFormatter<'_, 'a>| {
                dispatch_attribute_value(language, &part, text, f)
            };
            let Some(alias) = dispatch(JsInVueTemplate::ForAlias, v_for.alias, f) else {
                return false;
            };
            let Some(source) = dispatch(JsInVueTemplate::ForSource, v_for.source, f) else {
                return false;
            };
            let (alias, source) = (format_ir(alias), format_ir(source));
            write_quoted(&format_args!(alias, space(), v_for.operator, space(), source), f);
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

struct VFor<'a> {
    /// As written, without the parens.
    alias: &'a str,
    operator: &'static str,
    source: &'a str,
}

/// `v-for`'s value split as Vue's `parseForExpression`:
/// `alias in source` / `alias of source`, the alias in parens or not, up to three names.
/// `None` for a value Vue does not read so, which prints as written.
fn split_v_for(value: &str) -> Option<VFor<'_>> {
    // `/(.*?)\s+(in|of)\s+(.*)/s`: the first whitespace run with `in` / `of` and whitespace after it
    let bytes = value.as_bytes();
    let (alias, operator, source) = (0..bytes.len()).find_map(|i| {
        if !bytes[i].is_ascii_whitespace() || (i > 0 && bytes[i - 1].is_ascii_whitespace()) {
            return None;
        }
        let rest = value[i..].trim_ascii_start();
        let operator = ["in", "of"].into_iter().find(|operator| rest.starts_with(operator))?;
        let after = &rest[2..];
        after
            .starts_with(|c: char| c.is_ascii_whitespace())
            .then(|| (&value[..i], operator, after.trim_ascii()))
    })?;
    if source.is_empty() {
        return None;
    }

    // `/^\(|\)$/g`, then the iterators: `/,([^,\]}]*)(?:,([^,\]}]*))?$/`
    // `LHS.trim().replace(stripParensRE, '').trim()`, as Vue reads it
    let alias = alias.trim_ascii();
    let alias = alias.strip_prefix('(').unwrap_or(alias);
    let alias = alias.strip_suffix(')').unwrap_or(alias).trim_ascii();
    // The last one or two commas with no `]` / `}` after them
    let tail = alias.rfind([']', '}']).map_or(0, |end| end + 1);
    let mut commas = alias[tail..].rmatch_indices(',').map(|(i, _)| tail + i);
    let parts = match (commas.next(), commas.next()) {
        (None, _) => [alias, "", ""],
        (Some(last), None) => [alias[..last].trim_ascii(), alias[last + 1..].trim_ascii(), ""],
        (Some(last), Some(first)) => [
            alias[..first].trim_ascii(),
            alias[first + 1..last].trim_ascii(),
            alias[last + 1..].trim_ascii(),
        ],
    };
    // A name is missing before one that is not (`, i`)
    if parts[0].is_empty() || (parts[1].is_empty() && !parts[2].is_empty()) {
        return None;
    }
    Some(VFor { alias, operator, source })
}
