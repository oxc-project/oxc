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
        NsPrefixKind, PseudoClassSelector, PseudoClassSelectorArgKind, PseudoElementSelector,
        PseudoElementSelectorArgKind, SelectorList, SimpleSelector, TypeSelector, WqName,
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
                    write_combinator(combinator, true, false, f);
                    write!(f, " ");
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
        // Nth, Number, LanguageRangeList, TokenSeq, LessExtendList:
        // print the source verbatim (normalized below where needed).
        _ => {
            let span = to_span(kind.span());
            write!(f, text(source.text_for(&span)));
        }
    }
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
