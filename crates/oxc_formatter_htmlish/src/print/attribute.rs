//! Attributes: the list in the start tag (line breaks, `prettier-ignore-attribute`),
//! each one's name as written, then the value, which a few attributes get re-printed by their own grammar.

use std::{any::Any, borrow::Cow};

use cow_utils::CowUtils;

use oxc_formatter_core::{
    Format, FormatElement, arena_cow_str,
    builders::{
        group, hard_line_break, if_group_breaks, if_group_fits_on_line, indent, soft_block_indent,
        soft_line_break, soft_line_break_or_space, space, text,
    },
    spec::is_suppression_marker,
    write,
};
use oxc_htmlish_parser::ast::{Attribute, NodeId, NodeKind, Quote};

use super::{
    HtmlishFormatter,
    embed::{
        CssInHtmlStyleAttribute, JsInHtmlEventHandler, is_event_handler, write_attribute_value,
    },
    format_with,
    tag::opening_end_borrowed,
};
use crate::{
    context::HtmlishFormatContext,
    tree::{Tree, collapse_whitespace},
};

/// `<!-- prettier-ignore-attribute -->` right before the element: the attribute names it lists (all when none),
/// which print as written.
fn ignored_attributes<'a>(t: &Tree<'a>, id: NodeId) -> Option<&'a str> {
    let NodeKind::Comment(comment) = t.kind(t[id].prev?) else { return None };
    if comment.bogus {
        return None;
    }
    let content = comment.content.slice(t.source()).trim_ascii();
    let (marker, names) =
        content.split_once(|c: char| c.is_ascii_whitespace()).unwrap_or((content, ""));
    marker.strip_suffix("-attribute").is_some_and(is_suppression_marker).then_some(names)
}

pub fn write_attributes<'a>(t: &'a Tree<'a>, id: NodeId, f: &mut HtmlishFormatter<'_, 'a>) {
    let item = &t[id];
    let Some(el) = t.element(id) else { return };
    // `<br />`
    let slash = el.self_closing || el.void;
    if el.attributes.is_empty() {
        if slash {
            write!(f, space());
        }
        return;
    }
    let (single_attribute_per_line, bracket_same_line) =
        (f.options().single_attribute_per_line, f.options().bracket_same_line);

    let force_not_to_break = t.tag(id) == "script"
        && matches!(el.attributes.as_slice(), [only] if t.attribute_is(only.name.slice(t.source()), "src"))
        && item.first_child.is_none();
    let per_line = single_attribute_per_line && el.attributes.len() > 1;
    let ignored = ignored_attributes(t, id);
    let is_ignored = |name: &str| {
        ignored.is_some_and(|names| {
            names.is_empty() || names.split_ascii_whitespace().any(|n| t.attribute_is(name, n))
        })
    };

    write!(
        f,
        indent(&format_with(|f| {
            if force_not_to_break {
                write!(f, space());
            } else {
                write!(f, soft_line_break_or_space());
            }
            for (i, attribute) in el.attributes.iter().enumerate() {
                if i > 0 {
                    if per_line {
                        write!(f, hard_line_break());
                    } else {
                        write!(f, soft_line_break_or_space());
                    }
                }
                let name = attribute.name.slice(t.source());
                if is_ignored(name) {
                    write!(f, text(attribute.span.slice(t.source())));
                } else {
                    write_attribute(t, id, attribute, f);
                }
            }
        }))
    );

    let hug_end = opening_end_borrowed(t, id)
        || (item.self_closing && t[t.parent(id)].borrows.last_child_closing_end)
        || force_not_to_break;
    if hug_end || bracket_same_line {
        if slash {
            write!(f, space());
        }
    } else if slash {
        write!(f, soft_line_break_or_space());
    } else {
        write!(f, soft_line_break());
    }
}

fn write_attribute<'a>(
    t: &'a Tree<'a>,
    element: NodeId,
    attribute: &Attribute<'a>,
    f: &mut HtmlishFormatter<'_, 'a>,
) {
    let source = t.source();
    let name = attribute.name.slice(source);
    let is = |known| t.attribute_is(name, known);
    write!(f, text(t.attribute_name(element, name)));
    let Some(value) = attribute.value else { return };
    let quoted = value.quote != Quote::None;
    let value = value.span.slice(source);
    write!(f, "=");

    let embedded_in = f.context().embedded_in();
    if !quoted && embedded_in.is_some_and(|e| e.is_placeholder(value)) {
        write!(f, text(value));
        return;
    }
    // A placeholder would hide in the sorted class list, out of the parent's reach
    let tailwind = f.options().sort_tailwindcss.as_ref().is_some_and(|attributes| {
        is("class") || attributes.iter().any(|attribute| attribute == name)
    }) && !embedded_in.is_some_and(|e| e.has_placeholder(value));
    if tailwind && write_tailwind_classes(value, is("class") && embedded_in.is_none(), f) {
        return;
    }
    // `{{` may be a template's interpolation: those values print as written.
    // Prettier formats the values below only in a document of its own.
    let reformat = embedded_in.is_none() && !value.contains("{{");
    let embedded = if !reformat {
        None
    } else if is("style") {
        Some(("css", &CssInHtmlStyleAttribute as &dyn Any))
    } else if is_event_handler(name) {
        Some(("js", &JsInHtmlEventHandler as &dyn Any))
    } else {
        None
    };
    if let Some((language, parent_context)) = embedded
        && write_attribute_value(language, parent_context, unescaped(value, f), f)
    {
        return;
    }
    let tag = t.tag(element);
    // A class list that collapses to nothing prints as written;
    // the names are split at ASCII whitespace only (DIVERGENCES.md#class-ascii-whitespace).
    if is("class") && reformat && !value.trim_ascii().is_empty() {
        let names = collapse_whitespace(f.allocator(), unescaped(value, f));
        write!(f, ["\"", text(quoted_piece(names, f)), "\""]);
    } else if is("srcset")
        && matches!(tag, "img" | "source")
        && let Some(candidates) = parse_srcset(unescaped(value, f))
    {
        write_srcset(&candidates, f);
    } else if is("allow") && tag == "iframe" && reformat {
        write_permissions_policy(unescaped(value, f), f);
    } else {
        write!(f, AttributeValue(value));
    }
}

/// A class list for the session's Tailwind sorter, which collapses its whitespace unless told to preserve it;
/// `class` collapses it anyway when printed on its own (`collapse`).
/// Returns `false` for a value the sorter leaves alone (`{{`, whitespace only),
/// or holding a `"`, whose quote is chosen by content.
fn write_tailwind_classes<'a>(
    value: &'a str,
    collapse: bool,
    f: &mut HtmlishFormatter<'_, 'a>,
) -> bool {
    let value = unescaped(value, f);
    if value.contains('"') || value.contains("{{") || value.trim_ascii().is_empty() {
        return false;
    }
    let classes = if collapse { collapse_whitespace(f.allocator(), value) } else { value };
    let index = f.session().add_tailwind_class(classes.to_string());
    write!(f, "\"");
    f.write_element(FormatElement::TailwindClass(index));
    write!(f, "\"");
    true
}

/// A quoted attribute value: `"` unless the value holds more `"` than `'`, the chosen quote escaped.
/// Quote entities are read back first, so the choice does not depend on how the source quoted.
pub struct AttributeValue<'a>(pub &'a str);

impl<'a> Format<'a, HtmlishFormatContext<'a>> for AttributeValue<'a> {
    fn fmt(&self, f: &mut HtmlishFormatter<'_, 'a>) {
        let raw = unescaped(self.0, f);
        let doubles = raw.bytes().filter(|&b| b == b'"').count();
        let singles = raw.bytes().filter(|&b| b == b'\'').count();
        let (quote, escaped) = if doubles > singles { ("'", "&apos;") } else { ("\"", "&quot;") };
        let printed = arena_cow_str(&raw.cow_replace(quote, escaped), f);
        write!(f, [text(quote), text(printed), text(quote)]);
    }
}

/// `&apos;` and `&quot;` read back.
fn unescaped<'a>(value: &'a str, f: &HtmlishFormatter<'_, 'a>) -> &'a str {
    if !value.contains('&') {
        return value;
    }
    let apos = value.cow_replace("&apos;", "'");
    match apos.cow_replace("&quot;", "\"") {
        Cow::Borrowed(_) => arena_cow_str(&apos, f),
        Cow::Owned(both) => f.allocator().alloc_str(&both),
    }
}

/// `"` re-escaped for a value printed in `"`.
pub(super) fn quoted_piece<'a>(piece: &'a str, f: &HtmlishFormatter<'_, 'a>) -> &'a str {
    arena_cow_str(&piece.cow_replace('"', "&quot;"), f)
}

/// Prettier's `printExpand`: `content` on its own indented lines when the group breaks.
pub(super) fn write_expanded<'a>(
    content: &impl Format<'a, HtmlishFormatContext<'a>>,
    f: &mut HtmlishFormatter<'_, 'a>,
) {
    write!(f, ["\"", group(&soft_block_indent(content)), "\""]);
}

struct Candidate<'a> {
    url: &'a str,
    /// The descriptor's number as printed, and its unit (`w` / `h` / `x`).
    descriptor: Option<(String, &'a str)>,
}

/// One candidate per line when long, descriptors aligned on their decimal point.
fn write_srcset<'a>(candidates: &[Candidate<'a>], f: &mut HtmlishFormatter<'_, 'a>) {
    let max_url = candidates.iter().map(|c| c.url.len()).max().unwrap_or(0);
    let left = |number: &str| number.find('.').unwrap_or(number.len());
    let max_left =
        candidates.iter().filter_map(|c| c.descriptor.as_ref().map(|d| left(&d.0))).max();
    let max_left = max_left.unwrap_or(0);
    let padding: &'a str = f.allocator().alloc_str(&" ".repeat(max_url + 1 + max_left));
    write_expanded(
        &format_with(|f| {
            for (i, c) in candidates.iter().enumerate() {
                if i > 0 {
                    write!(f, [",", soft_line_break_or_space()]);
                }
                let url = quoted_piece(c.url, f);
                write!(f, text(url));
                if let Some((number, unit)) = &c.descriptor {
                    let padding = &padding[..max_url - c.url.len() + 1 + max_left - left(number)];
                    let number = f.allocator().alloc_str(number);
                    write!(
                        f,
                        [
                            if_group_breaks(&text(padding)),
                            if_group_fits_on_line(&space()),
                            text(number),
                            text(unit)
                        ]
                    );
                }
            }
        }),
        f,
    );
}

/// `@prettier/parse-srcset`: `None` for what it rejects (a bad descriptor, or no candidate),
/// and for mixed descriptor kinds, which Prettier does not print either.
fn parse_srcset(input: &str) -> Option<Vec<Candidate<'_>>> {
    let is_ws = |c: char| c.is_ascii_whitespace();
    let mut candidates = vec![];
    let mut kind = None;
    let mut rest = input;
    loop {
        rest = rest.trim_start_matches(|c| is_ws(c) || c == ',');
        if rest.is_empty() {
            break;
        }
        let url_end = rest.find(is_ws).unwrap_or(rest.len());
        let mut url = &rest[..url_end];
        rest = &rest[url_end..];
        let descriptors = if url.ends_with(',') {
            url = url.trim_end_matches(',');
            ""
        } else {
            // Descriptors up to the next `,` outside parentheses
            rest = rest.trim_ascii_start();
            let mut depth = 0;
            let mut end = rest.len();
            for (i, c) in rest.char_indices() {
                match c {
                    '(' => depth += 1,
                    ')' if depth > 0 => depth -= 1,
                    ',' if depth == 0 => {
                        end = i;
                        break;
                    }
                    _ => {}
                }
            }
            let descriptors = &rest[..end];
            rest = rest.get(end + 1..).unwrap_or("");
            descriptors
        };
        let mut descriptor = None;
        for d in descriptors.split_ascii_whitespace() {
            let unit = d.chars().last()?;
            let value = &d[..d.len() - unit.len_utf8()];
            let parsed = match unit {
                'w' | 'h' if value.bytes().all(|b| b.is_ascii_digit()) => {
                    value.parse::<f64>().ok().filter(|&v| v > 0.0)
                }
                // A zero density is kept (DIVERGENCES.md#srcset-zero-density)
                'x' if is_floating_point(value) => value.parse::<f64>().ok().filter(|&v| v >= 0.0),
                _ => None,
            }?;
            let parsed = zmij_ecma::Buffer::new().format(parsed).to_owned();
            // One descriptor per candidate and one kind per list:
            // a second is an error to Prettier or a mix it rejects.
            if descriptor.is_some() || kind.is_some_and(|kind| kind != unit) {
                return None;
            }
            kind = Some(unit);
            descriptor = Some((parsed, &d[d.len() - unit.len_utf8()..]));
        }
        candidates.push(Candidate { url, descriptor });
    }
    (!candidates.is_empty()).then_some(candidates)
}

/// `/^-?(?:[0-9]+|[0-9]*\.[0-9]+)(?:[eE][+-]?[0-9]+)?$/`
fn is_floating_point(value: &str) -> bool {
    let value = value.strip_prefix('-').unwrap_or(value);
    let (mantissa, exponent) = match value.find(['e', 'E']) {
        Some(i) => (&value[..i], Some(&value[i + 1..])),
        None => (value, None),
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    let mantissa_ok = match mantissa.split_once('.') {
        Some((int, frac)) => (int.is_empty() || digits(int)) && digits(frac),
        None => digits(mantissa),
    };
    let exponent_ok = exponent.is_none_or(|e| digits(e.strip_prefix(['+', '-']).unwrap_or(e)));
    mantissa_ok && exponent_ok
}

/// `allow` (permissions policy): one directive per line when long, `;` between them.
fn write_permissions_policy<'a>(value: &'a str, f: &mut HtmlishFormatter<'_, 'a>) {
    let directives: Vec<&'a str> = value
        .split(';')
        .map(|token| collapse_whitespace(f.allocator(), token))
        .filter(|directive| !directive.is_empty())
        .map(|directive| quoted_piece(directive, f))
        .collect();
    if directives.is_empty() {
        write!(f, "\"\"");
        return;
    }
    write_expanded(
        &format_with(|f| {
            for (i, directive) in directives.iter().enumerate() {
                write!(f, text(directive));
                if i + 1 == directives.len() {
                    write!(f, if_group_breaks(&text(";")));
                } else {
                    write!(f, [";", soft_line_break_or_space()]);
                }
            }
        }),
        f,
    );
}
