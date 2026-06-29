//! Selector printing. Ports Prettier's `selector-*` cases (postcss-selector-parser)
//! onto raffia's structured selector AST.

use std::borrow::Cow;

use cow_utils::CowUtils;
use oxc_formatter_core::{
    Buffer,
    builders::{group, hard_line_break, indent, soft_line_break, soft_line_break_or_space, text},
    write,
};
use raffia::{
    Spanned,
    ast::{
        AttributeSelector, AttributeSelectorMatcherKind, AttributeSelectorValue, Combinator,
        CombinatorKind, ComplexSelector, ComplexSelectorChild, CompoundSelector, InterpolableIdent,
        NsPrefixKind, Nth, NthIndex, PseudoClassSelector, PseudoClassSelectorArgKind,
        PseudoElementSelector, PseudoElementSelectorArgKind, SelectorList, SimpleSelector,
        TypeSelector, WqName,
    },
};

use crate::{
    format::to_span,
    print::{CssFormatter, format_with, statement::write_maybe_lowercase},
};

/// How a selector list separates its selectors.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SelectorListStyle {
    /// `,` + hardline (rule selectors at any nesting level).
    Hard,
    /// `,` + line (inside `@extend`, `@custom-selector`, `@nest`, pseudo args).
    Line,
}

/// Mirrors Prettier's `selector-root`.
pub fn write_selector_list<'a>(
    list: &SelectorList<'a>,
    style: SelectorListStyle,
    f: &mut CssFormatter<'_, 'a>,
) {
    // css-in-js `${}` placeholders (raffia fork's
    // `tolerate_at_keyword_placeholders`): postcss-selector-parser degrades
    // on at-words — everything from the selector containing the first
    // placeholder onwards becomes one garbage token soup whose commas no
    // longer split selectors. Prettier then prints it near-verbatim:
    // whitespace runs collapse to single spaces and the line never breaks.
    // Selectors BEFORE the first placeholder still split normally.
    // Only the embedded entry point can contain placeholders; the gate also
    // keeps a literal marker inside e.g. an attribute value (`[x="@prettier"]`)
    // in a standalone file from triggering garbage mode.
    let placeholder_idx = if f.context().template_placeholders() {
        let source = f.context().source_text();
        list.selectors.iter().position(|complex| {
            source.text_for(&to_span(complex.span())).contains(crate::TEMPLATE_PLACEHOLDER_PREFIX)
        })
    } else {
        None
    };

    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        for (i, complex) in list.selectors.iter().enumerate() {
            if i > 0 {
                write!(f, ",");
                // A comment trailing the comma stays on the same line.
                let next_start = to_span(complex.span()).start;
                if let Some(comment) = f.context().comments().peek()
                    && comment.span.end <= next_start
                {
                    let source = f.context().source_text();
                    let prev_end = to_span(list.selectors[i - 1].span()).end;
                    if crate::comments::classify_gap(
                        source.bytes_range(prev_end, comment.span.start),
                    ) == crate::comments::Gap::None
                    {
                        f.context().comments().take_before(comment.span.end);
                        write!(f, " ");
                        crate::comments::write_single_comment(comment, f);
                    }
                }
                match style {
                    SelectorListStyle::Hard => write!(f, hard_line_break()),
                    SelectorListStyle::Line => write!(f, soft_line_break_or_space()),
                }
            }
            // Comments on their own line between selectors.
            let start = to_span(complex.span()).start;
            for &comment in f.context().comments().take_before(start) {
                crate::comments::write_single_comment(comment, f);
                write!(f, hard_line_break());
            }
            if placeholder_idx == Some(i) {
                let source = f.context().source_text();
                let end = to_span(list.selectors[list.selectors.len() - 1].span()).end;
                let raw = source.slice_range(start, end);
                let mut collapsed = oxc_allocator::StringBuilder::new_in(f.allocator());
                for (k, word) in raw.split_ascii_whitespace().enumerate() {
                    if k > 0 {
                        collapsed.push_str(" ");
                    }
                    collapsed.push_str(word);
                }
                // Comments inside the chunk are part of the verbatim text.
                let _ = f.context().comments().take_before(end);
                write!(f, text(collapsed.into_str()));
                return;
            }
            write_complex_selector(complex, f);
        }
    });
    write!(f, group(&body));
}

/// Mirrors Prettier's `selector-selector`: group, indented when long
/// (more than 2 children).
pub fn write_complex_selector<'a>(complex: &ComplexSelector<'a>, f: &mut CssFormatter<'_, 'a>) {
    let should_indent = complex.children.len() > 2;
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        for (i, child) in complex.children.iter().enumerate() {
            match child {
                ComplexSelectorChild::CompoundSelector(compound) => {
                    write_compound_selector(compound, f);
                }
                ComplexSelectorChild::Combinator(combinator) => {
                    write_combinator(combinator, i == 0, i + 1 == complex.children.len(), f);
                }
            }
        }
    });
    if should_indent {
        write!(f, group(&indent(&body)));
    } else {
        write!(f, group(&body));
    }
}

/// Mirrors Prettier's `selector-combinator`:
/// breakable line BEFORE the combinator, space after.
fn write_combinator(
    combinator: &Combinator,
    is_first: bool,
    is_last: bool,
    f: &mut CssFormatter<'_, '_>,
) {
    match &combinator.kind {
        CombinatorKind::Descendant => {
            write!(f, soft_line_break_or_space());
        }
        kind => {
            if !is_first {
                write!(f, soft_line_break_or_space());
            }
            match kind {
                CombinatorKind::Child => write!(f, ">"),
                CombinatorKind::NextSibling => write!(f, "+"),
                CombinatorKind::LaterSibling => write!(f, "~"),
                CombinatorKind::Column => write!(f, "||"),
                CombinatorKind::Descendant => unreachable!(),
            }
            if !is_last {
                write!(f, " ");
            }
        }
    }
}

pub fn write_compound_selector<'a>(compound: &CompoundSelector<'a>, f: &mut CssFormatter<'_, 'a>) {
    for child in &compound.children {
        write_simple_selector(child, f);
    }
}

fn write_simple_selector<'a>(selector: &SimpleSelector<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    match selector {
        SimpleSelector::Class(class) => {
            write!(f, ".");
            write_interpolable_ident(&class.name, f);
        }
        SimpleSelector::Id(id) => {
            write!(f, "#");
            write_interpolable_ident(&id.name, f);
        }
        SimpleSelector::Type(type_selector) => write_type_selector(type_selector, f),
        SimpleSelector::Attribute(attribute) => write_attribute_selector(attribute, f),
        SimpleSelector::PseudoClass(pseudo) => write_pseudo_class(pseudo, f),
        SimpleSelector::PseudoElement(pseudo) => write_pseudo_element(pseudo, f),
        SimpleSelector::Nesting(nesting) => {
            write!(f, "&");
            if let Some(suffix) = &nesting.suffix {
                write_interpolable_ident(suffix, f);
            }
        }
        SimpleSelector::SassPlaceholder(placeholder) => {
            let span = to_span(placeholder.span());
            write!(f, text(source.text_for(&span)));
        }
    }
}

fn write_interpolable_ident<'a>(ident: &InterpolableIdent<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let span = to_span(ident.span());
    let raw = source.text_for(&span);
    // Multi-line interpolations collapse to single spaces.
    if raw.contains('\n') || raw.contains('\r') {
        let joined = raw.split_whitespace().collect::<Vec<_>>().join(" ");
        write!(f, text(f.allocator().alloc_str(&joined)));
    } else {
        write!(f, text(raw));
    }
}

fn write_wq_name<'a>(name: &WqName<'a>, f: &mut CssFormatter<'_, 'a>) {
    if let Some(prefix) = &name.prefix {
        if let Some(kind) = &prefix.kind {
            match kind {
                NsPrefixKind::Ident(ident) => write_interpolable_ident(ident, f),
                NsPrefixKind::Universal(_) => write!(f, "*"),
            }
        }
        write!(f, "|");
    }
    write_interpolable_ident(&name.name, f);
}

/// Mirrors Prettier's `selector-tag`: lowercase `from`/`to` inside
/// `@keyframes`; HTML tag names are printed as-is (adjustNumbers is a no-op
/// for plain identifiers).
fn write_type_selector<'a>(type_selector: &TypeSelector<'a>, f: &mut CssFormatter<'_, 'a>) {
    match type_selector {
        TypeSelector::TagName(tag) => write_wq_name(&tag.name, f),
        TypeSelector::Universal(universal) => {
            if let Some(prefix) = &universal.prefix {
                if let Some(kind) = &prefix.kind {
                    match kind {
                        NsPrefixKind::Ident(ident) => write_interpolable_ident(ident, f),
                        NsPrefixKind::Universal(_) => write!(f, "*"),
                    }
                }
                write!(f, "|");
            }
            write!(f, "*");
        }
    }
}

/// Mirrors Prettier's `selector-attribute`.
fn write_attribute_selector<'a>(attribute: &AttributeSelector<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    write!(f, "[");
    write_wq_name(&attribute.name, f);
    if let Some(matcher) = &attribute.matcher {
        match matcher.kind {
            AttributeSelectorMatcherKind::Exact => write!(f, "="),
            AttributeSelectorMatcherKind::MatchWord => write!(f, "~="),
            AttributeSelectorMatcherKind::ExactOrPrefixThenHyphen => write!(f, "|="),
            AttributeSelectorMatcherKind::Prefix => write!(f, "^="),
            AttributeSelectorMatcherKind::Suffix => write!(f, "$="),
            AttributeSelectorMatcherKind::Substring => write!(f, "*="),
        }
    }
    if let Some(value) = &attribute.value {
        match value {
            // Unquoted values get quoted (Prettier's `quoteAttributeValue`).
            AttributeSelectorValue::Ident(ident) => {
                let quote = if f.options().single_quote.value() { "'" } else { "\"" };
                let span = to_span(ident.span());
                write!(f, [text(quote), text(source.text_for(&span)), text(quote)]);
            }
            AttributeSelectorValue::Str(raffia::ast::InterpolableStr::Literal(str)) => {
                crate::print::value::write_str(str, f);
            }
            _ => {
                let span = to_span(value.span());
                write!(f, text(source.text_for(&span)));
            }
        }
    }
    if let Some(modifier) = &attribute.modifier {
        write!(f, " ");
        let span = to_span(modifier.ident.span());
        write!(f, text(source.text_for(&span)));
    }
    write!(f, "]");
}

/// Mirrors Prettier's `selector-pseudo`.
fn write_pseudo_class<'a>(pseudo: &PseudoClassSelector<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    write!(f, ":");
    let name_span = to_span(pseudo.name.span());
    write_maybe_lowercase(source.text_for(&name_span), f);
    if let Some(arg) = &pseudo.arg {
        let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            write!(f, soft_line_break());
            write_pseudo_class_arg(&arg.kind, f);
        });
        write!(
            f,
            group(&format_with(move |f: &mut CssFormatter<'_, 'a>| {
                write!(f, "(");
                write!(f, indent(&body));
                write!(f, soft_line_break());
                write!(f, ")");
            }))
        );
    }
}

fn write_pseudo_class_arg<'a>(kind: &PseudoClassSelectorArgKind<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    match kind {
        PseudoClassSelectorArgKind::SelectorList(list) => {
            write_selector_list_inline(list, f);
        }
        PseudoClassSelectorArgKind::RelativeSelectorList(list) => {
            for (i, selector) in list.selectors.iter().enumerate() {
                if i > 0 {
                    write!(f, ",");
                    write!(f, soft_line_break_or_space());
                }
                if let Some(combinator) = &selector.combinator {
                    // `is_last: false` already appends the space after `>`.
                    write_combinator(combinator, true, false, f);
                }
                write_complex_selector(&selector.complex_selector, f);
            }
        }
        PseudoClassSelectorArgKind::CompoundSelector(compound) => {
            write_compound_selector(compound, f);
        }
        PseudoClassSelectorArgKind::CompoundSelectorList(list) => {
            for (i, compound) in list.selectors.iter().enumerate() {
                if i > 0 {
                    write!(f, ",");
                    write!(f, soft_line_break_or_space());
                }
                write_compound_selector(compound, f);
            }
        }
        PseudoClassSelectorArgKind::Ident(ident) => write_interpolable_ident(ident, f),
        PseudoClassSelectorArgKind::Nth(nth) => write_nth(nth, f),
        // Number, LanguageRangeList, TokenSeq, LessExtendList:
        // print the source verbatim (normalized below where needed).
        _ => {
            let span = to_span(kind.span());
            write!(f, text(source.text_for(&span)));
        }
    }
}

/// `:nth-child()` argument. postcss-selector-parser tokenizes `An+B` so that
/// a `+` becomes a combinator (printed with one space on both sides) while a
/// `-` stays glued inside the word; digit-led words land in
/// `selector-unknown` and get `maybeToLowerCase`d, letter-led ones are tags
/// and keep their case. `odd`/`even`/integers print verbatim.
fn write_nth<'a>(nth: &Nth<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let index_span = to_span(nth.index.span());
    let raw = source.text_for(&index_span);
    match &nth.index {
        NthIndex::AnPlusB(_) => {
            let normalized = normalize_an_plus_b(raw);
            match normalized {
                Cow::Borrowed(s) => write!(f, text(s)),
                Cow::Owned(s) => write!(f, text(f.allocator().alloc_str(&s))),
            }
        }
        NthIndex::Odd(_) | NthIndex::Even(_) | NthIndex::Integer(_) => {
            write!(f, text(raw));
        }
    }
    if let Some(matcher) = &nth.matcher {
        write!(f, " ");
        let matcher_span = to_span(matcher.span());
        if let Some(selector) = &matcher.selector {
            // The `of` keyword as written, then the selector list.
            let keyword_end = to_span(selector.span()).start;
            let keyword = source.slice_range(matcher_span.start, keyword_end).trim_ascii_end();
            write!(f, [text(keyword), " "]);
            write_selector_list_inline(selector, f);
        } else {
            write!(f, text(source.text_for(&matcher_span)));
        }
    }
}

/// Normalize an `An+B` expression: exactly one space around each `+`
/// (none before a leading `+`), digit-led segments lowercased, all other
/// whitespace kept as-is (a glued `-` stays glued).
fn normalize_an_plus_b(raw: &str) -> Cow<'_, str> {
    let bytes = raw.as_bytes();
    if !bytes.iter().any(|&b| b == b'+' || b.is_ascii_uppercase()) {
        return Cow::Borrowed(raw);
    }

    let mut out = String::with_capacity(raw.len() + 4);
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'+' {
            while out.ends_with(' ') {
                out.pop();
            }
            if !out.is_empty() {
                out.push(' ');
            }
            out.push('+');
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() {
                out.push(' ');
            }
        } else if b.is_ascii_whitespace() {
            out.push(b as char);
            i += 1;
        } else {
            let start = i;
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'+' {
                i += 1;
            }
            let segment = &raw[start..i];
            if segment.as_bytes()[0].is_ascii_digit() {
                for c in segment.chars() {
                    out.push(c.to_ascii_lowercase());
                }
            } else {
                out.push_str(segment);
            }
        }
    }
    Cow::Owned(out)
}

/// Selector list inside pseudo args: `,` + line.
fn write_selector_list_inline<'a>(list: &SelectorList<'a>, f: &mut CssFormatter<'_, 'a>) {
    for (i, complex) in list.selectors.iter().enumerate() {
        if i > 0 {
            write!(f, ",");
            write!(f, soft_line_break_or_space());
        }
        write_complex_selector(complex, f);
    }
}

fn write_pseudo_element<'a>(pseudo: &PseudoElementSelector<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    write!(f, "::");
    let name_span = to_span(pseudo.name.span());
    write_maybe_lowercase(source.text_for(&name_span), f);
    if let Some(arg) = &pseudo.arg {
        write!(f, "(");
        match &arg.kind {
            PseudoElementSelectorArgKind::CompoundSelector(compound) => {
                write_compound_selector(compound, f);
            }
            PseudoElementSelectorArgKind::Ident(ident) => write_interpolable_ident(ident, f),
            PseudoElementSelectorArgKind::TokenSeq(seq) => {
                let span = to_span(seq.span());
                write!(f, text(source.text_for(&span)));
            }
        }
        write!(f, ")");
    }
}

/// Lowercase `from` / `to` keyframe selectors; keep percentages as numbers.
pub fn write_keyframe_selector<'a>(
    selector: &raffia::ast::KeyframeSelector<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    match selector {
        raffia::ast::KeyframeSelector::Ident(ident) => {
            let span = to_span(ident.span());
            let raw = source.text_for(&span);
            match raw.cow_to_ascii_lowercase() {
                Cow::Borrowed(s) => write!(f, text(s)),
                Cow::Owned(s) => write!(f, text(f.allocator().alloc_str(&s))),
            }
        }
        raffia::ast::KeyframeSelector::Percentage(percentage) => {
            crate::print::value::write_number(&percentage.value, f);
            write!(f, "%");
        }
    }
}
