//! Vue: the JS of `{{ }}` and directive values, formatted by the session's dispatcher as fragments
//! in the file's script language. A part the dispatcher does not format prints as written.

use oxc_allocator::ArenaVec;
use oxc_formatter_core::{
    FormatElement,
    builders::{space, text},
    dispatch_fragment_ir, format_args,
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
    /// `v-for`'s `aliases in source`; `None` for a value Vue does not read so.
    For(Option<VueFor>),
    /// An event handler's: an expression, or statements.
    Handler,
}

/// `None` for an attribute whose value is no JS.
pub fn value_kind(t: &Tree<'_>, element: NodeId, attribute: &Attribute<'_>) -> Option<Value> {
    let source = t.source();
    let name = attribute.name.slice(source);
    let AttributeKind::Vue(directive) = &attribute.kind else {
        // `slot-scope`, `setup` and `vars` are plain attributes (apps/oxfmt/DIVERGENCES.md#js-in-vue-removed-attributes-as-written)
        return (name == "generic" && t.is_sfc_block(element) && t.tag(element) == "script")
            .then_some(Value::Fragment(JsInVueTemplate::ScriptGeneric));
    };
    // Tailwind classes (`Tree::tailwind`): a bound one without modifiers, a listed directive as written
    let tailwind = t.tailwind();
    let bound_classes = || {
        tailwind.is_some_and(|tailwind| {
            directive.modifiers.is_empty()
                && directive
                    .argument
                    .filter(|argument| !argument.dynamic)
                    .is_some_and(|argument| tailwind.holds(argument.span.slice(source)))
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
            tailwind_classes: tailwind.is_some_and(|tailwind| tailwind.directive(name)),
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

/// `{{ }}`'s expression formatted; `None` to print it as written.
pub fn dispatch_interpolation<'a>(
    t: &Tree<'_>,
    expression: &str,
    f: &HtmlishFormatter<'_, 'a>,
) -> Option<ArenaVec<'a, FormatElement<'a>>> {
    // The code Vue compiles, trimmed
    let code = expression.trim_ascii();
    dispatch_fragment_ir(f, t.vue_script_language(), code, Some(&JsInVueTemplate::Interpolation))
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
