//! What formatting must not change, as one string per document:
//! elements (name, attributes, end tag presence; names in any case where known ones print lowercase),
//! comments as written, a doctype in any case,
//! attribute values with their quote entities read back (a class list as its names, `srcset` as its URLs, `allow` as its directives),
//! and text as its words (whitespace runs may be re-flowed), except where whitespace is content:
//! `pre` as written, a `script` / `style` body as its lines (re-indented, blank ones at its ends dropped).

use std::{borrow::Cow, fmt::Write as _};

use cow_utils::CowUtils;

use oxc_htmlish_parser::ast::{Namespace, NodeId, NodeKind, Root};

/// `fold_names`: a `.html` file, where known names print lowercase (DIVERGENCES.md#tag-name-case).
pub fn fingerprint(source: &str, root: &Root<'_>, fold_names: bool) -> String {
    let mut out = String::new();
    walk(source, root, NodeId::DOCUMENT, Text::Words, fold_names, &mut out);
    out
}

/// How a text keeps its whitespace.
#[derive(Clone, Copy)]
enum Text {
    Words,
    Lines,
    AsWritten,
}

fn walk(
    source: &str,
    root: &Root<'_>,
    parent: NodeId,
    text: Text,
    fold_names: bool,
    out: &mut String,
) {
    for (id, node) in root.children(parent) {
        match &node.kind {
            NodeKind::Element(el) => {
                let name = fold(el.name.slice(source), fold_names);
                write!(out, "<{name}").unwrap();
                for attribute in &el.attributes {
                    let attribute_name = attribute.name.slice(source);
                    write!(out, " {}", fold(attribute_name, fold_names)).unwrap();
                    let Some(value) = attribute.value else { continue };
                    let value = value.span.slice(source).cow_replace("&quot;", "\"");
                    let value = value.cow_replace("&apos;", "'");
                    let is = |name: &str| attribute_name.eq_ignore_ascii_case(name);
                    out.push('=');
                    if is("class") {
                        for name in value.split_ascii_whitespace() {
                            write!(out, " {name:?}").unwrap();
                        }
                    } else if is("srcset") {
                        // The URLs; descriptors are re-printed as numbers.
                        for candidate in value.split(',') {
                            if let Some(url) = candidate.split_ascii_whitespace().next() {
                                write!(out, " {url:?}").unwrap();
                            }
                        }
                    } else if is("allow") {
                        for directive in value.split(';') {
                            let words: Vec<&str> = directive.split_ascii_whitespace().collect();
                            if !words.is_empty() {
                                write!(out, " {:?}", words.join(" ")).unwrap();
                            }
                        }
                    } else {
                        write!(out, "{value:?}").unwrap();
                    }
                }
                out.push('>');
                // The formatter's `pre_like` and `script_like`, restated rather than shared
                // so that the check stays independent of the code under test.
                let text = if el.namespace != Namespace::Svg
                    && matches!(el.key, "pre" | "textarea" | "listing" | "plaintext" | "xmp")
                {
                    Text::AsWritten
                } else if matches!(el.key, "script" | "style") {
                    Text::Lines
                } else {
                    text
                };
                walk(source, root, id, text, fold_names, out);
                if el.end_tag.is_some() {
                    write!(out, "</{name}>").unwrap();
                }
            }
            NodeKind::Text(_) => {
                let value = node.span.slice(source);
                match text {
                    Text::AsWritten => write!(out, "{value:?}").unwrap(),
                    Text::Lines => {
                        for line in value.trim_ascii().lines() {
                            write!(out, "{:?}", line.trim_ascii()).unwrap();
                        }
                    }
                    Text::Words => {
                        for word in value.split_ascii_whitespace() {
                            write!(out, " {word}").unwrap();
                        }
                    }
                }
            }
            NodeKind::ConditionalComment(comment) => {
                write!(out, "[cc {}]", words(comment.condition.slice(source))).unwrap();
                if comment.parsed {
                    walk(source, root, id, text, fold_names, out);
                } else {
                    write!(out, "{:?}", comment.content.slice(source)).unwrap();
                }
            }
            // The expression's tokens may be re-flowed until it is formatted.
            NodeKind::Interpolation(interpolation) => {
                write!(out, "{{{{{}}}}}", words(interpolation.expression.slice(source))).unwrap();
            }
            NodeKind::Block(block) => {
                write!(out, "[@{}", words(block.name.slice(source))).unwrap();
                for parameter in &block.parameters {
                    write!(out, " ({})", words(parameter.slice(source))).unwrap();
                }
                out.push('{');
                walk(source, root, id, text, fold_names, out);
                out.push_str(if block.close.is_some() { "}]" } else { "]" });
            }
            // HTML reads a doctype ASCII case-insensitively
            NodeKind::DocType(_) => {
                write!(out, "[{:?}]", words(&node.span.slice(source).cow_to_ascii_lowercase()))
                    .unwrap();
            }
            _ => write!(out, "[{:?}]", node.span.slice(source)).unwrap(),
        }
    }
}

/// `name`, ASCII-lowercased with `fold_names`.
fn fold(name: &str, fold_names: bool) -> Cow<'_, str> {
    if fold_names { name.cow_to_ascii_lowercase() } else { Cow::Borrowed(name) }
}

/// `text`'s words, one space apart.
fn words(text: &str) -> String {
    text.split_ascii_whitespace().collect::<Vec<_>>().join(" ")
}
