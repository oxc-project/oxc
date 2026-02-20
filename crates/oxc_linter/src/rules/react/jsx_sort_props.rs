use std::cmp::Ordering;

use cow_utils::CowUtils;
use icu_collator::options::{CollatorOptions, Strength};
use icu_collator::{Collator, CollatorBorrowed};
use icu_locale::Locale;
use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeName, JSXElementName, JSXOpeningElement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

fn callbacks_last_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Callbacks must be listed after all other props").with_label(span)
}

fn shorthand_first_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Shorthand props must be listed before all other props").with_label(span)
}

fn shorthand_last_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Shorthand props must be listed after all other props").with_label(span)
}

fn multiline_first_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Multiline props must be listed before all other props").with_label(span)
}

fn multiline_last_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Multiline props must be listed after all other props").with_label(span)
}

fn reserved_first_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Reserved props must be listed before all other props").with_label(span)
}

fn sort_first_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Props in sortFirst must be listed before all other props").with_label(span)
}

fn alphabetical_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Props should be sorted alphabetically").with_label(span)
}

fn reserved_first_empty_list_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("A customized reserved first list must not be empty").with_label(span)
}

fn reserved_first_invalid_props_diagnostic(invalid_words: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "A customized reserved first list must only contain a subset of React reserved props. Remove: {invalid_words}"
    ))
    .with_label(span)
}

const ALL_RESERVED_PROPS: &[&str] = &["children", "dangerouslySetInnerHTML", "key", "ref"];

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum MultilineOption {
    #[default]
    Ignore,
    First,
    Last,
}

/// The `reservedFirst` option can be a boolean or an array of strings.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
enum ReservedFirst {
    Bool(bool),
    Array(Vec<String>),
}

impl Default for ReservedFirst {
    fn default() -> Self {
        ReservedFirst::Bool(false)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct JsxSortPropsConfig {
    callbacks_last: bool,
    shorthand_first: bool,
    shorthand_last: bool,
    ignore_case: bool,
    no_sort_alphabetically: bool,
    multiline: MultilineOption,
    reserved_first: ReservedFirst,
    sort_first: Vec<String>,
    locale: String,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct JsxSortProps(Box<JsxSortPropsConfig>);

impl std::ops::Deref for JsxSortProps {
    type Target = JsxSortPropsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// Enforce props alphabetical sorting in JSX elements.
    ///
    /// ### Why is this bad?
    /// Unsorted props can make components harder to read and maintain.
    /// Consistent prop ordering improves code readability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <App z a />;
    /// <App b a c />;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <App a z />;
    /// <App a b c />;
    /// ```
    JsxSortProps,
    react,
    style,
    fix,
    config = JsxSortPropsConfig,
);

impl Rule for JsxSortProps {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_elem) = node.kind() else {
            return;
        };

        let config = &*self.0;

        // Validate reservedFirst config and determine the effective reserved list.
        let reserved_list: Vec<&str> = match &config.reserved_first {
            ReservedFirst::Bool(true) => {
                let mut list = ALL_RESERVED_PROPS.to_vec();
                // dangerouslySetInnerHTML is only reserved on DOM components
                if !is_dom_component(jsx_opening_elem) {
                    list.retain(|&p| p != "dangerouslySetInnerHTML");
                }
                list
            }
            ReservedFirst::Array(arr) => {
                if arr.is_empty() {
                    // Empty array → report error on every prop
                    for attr in &jsx_opening_elem.attributes {
                        ctx.diagnostic(reserved_first_empty_list_diagnostic(attr_span(attr)));
                    }
                    return;
                }
                // Check for invalid (non-reserved) entries
                let invalid: Vec<&str> = arr
                    .iter()
                    .filter(|s| !ALL_RESERVED_PROPS.contains(&s.as_str()))
                    .map(String::as_str)
                    .collect();
                if !invalid.is_empty() {
                    let invalid_words = invalid.join(", ");
                    for attr in &jsx_opening_elem.attributes {
                        ctx.diagnostic(reserved_first_invalid_props_diagnostic(
                            &invalid_words,
                            attr_span(attr),
                        ));
                    }
                    return;
                }
                let mut list: Vec<&str> = arr
                    .iter()
                    .filter_map(|s| {
                        let s_str = s.as_str();
                        if ALL_RESERVED_PROPS.contains(&s_str) { Some(s_str) } else { None }
                    })
                    .collect();
                // dangerouslySetInnerHTML is only reserved on DOM components
                if !is_dom_component(jsx_opening_elem) {
                    list.retain(|&p| p != "dangerouslySetInnerHTML");
                }
                list
            }
            ReservedFirst::Bool(false) => vec![],
        };

        // Build a locale-aware collator when an explicit locale is configured.
        // Matches ESLint behavior: "auto" or empty uses byte-order comparison,
        // an explicit locale (e.g. "sk-SK") uses ICU collation.
        let collator = create_collator(&config.locale, config.ignore_case);

        // Collect groups of attributes separated by spread attributes.
        let groups = collect_groups(&jsx_opening_elem.attributes);

        for group in &groups {
            if group.len() < 2 {
                continue;
            }

            let prop_infos: Vec<PropInfo> = group
                .iter()
                .map(|&attr| classify_prop(attr, config, &reserved_list, ctx))
                .collect();

            // ── Comment-aware sorting ───────────────────────────────
            // Build a per-attribute map that extends each attribute's
            // source range to include trailing comments (and possibly a
            // consumed next attribute), following ESLint's algorithm.
            let cg = compute_comment_grouping(group, jsx_opening_elem.span.end, ctx);

            // Sortable items are those NOT consumed by a preceding attribute.
            let sortable_idx: Vec<usize> =
                cg.iter().enumerate().filter(|(_, c)| !c.consumed).map(|(i, _)| i).collect();

            if sortable_idx.len() < 2 {
                continue;
            }

            // Sort the sortable items with has_comment awareness:
            // items with has_comment go to the END of the group.
            let mut sorted_order: Vec<usize> = (0..sortable_idx.len()).collect();
            sorted_order.sort_by(|&a, &b| {
                let ai = sortable_idx[a];
                let bi = sortable_idx[b];
                match (cg[ai].has_comment, cg[bi].has_comment) {
                    (true, false) => Ordering::Greater,
                    (false, true) => Ordering::Less,
                    _ => compare_props(&prop_infos[ai], &prop_infos[bi], config, collator.as_ref()),
                }
            });

            // Check if the group is already sorted.
            let is_sorted = sorted_order.iter().enumerate().all(|(pos, &orig)| pos == orig);
            if is_sorted {
                continue;
            }

            // Build the original / sorted PropInfo slices for diagnostic
            // selection (operates on sortable items only).
            let original_sortable: Vec<PropInfo> =
                sortable_idx.iter().map(|&i| prop_infos[i].clone()).collect();
            let sorted_sortable: Vec<PropInfo> =
                sorted_order.iter().map(|&pos| prop_infos[sortable_idx[pos]].clone()).collect();

            let diagnostic_fn = find_first_violation(&original_sortable, &sorted_sortable, config);

            // Determine the replacement span — from first sortable attr to
            // the furthest extended_end among all sortable items.
            let first_start = attr_span(group[*sortable_idx.first().unwrap()]).start;
            let max_ext_end = sortable_idx.iter().map(|&i| cg[i].extended_end).max().unwrap();
            let group_span = Span::new(first_start, max_ext_end);

            let diagnostic = diagnostic_fn(group_span);

            // Check whether any unhandled comments remain in the gaps
            // between sortable items (i.e. comments that were NOT absorbed
            // into an extended range).  If so, we cannot safely reorder.
            let has_unhandled_comments = (0..sortable_idx.len().saturating_sub(1)).any(|i| {
                let curr_ext = cg[sortable_idx[i]].extended_end;
                let next_start = attr_span(group[sortable_idx[i + 1]]).start;
                curr_ext < next_start && ctx.has_comments_between(Span::new(curr_ext, next_start))
            });

            if has_unhandled_comments {
                ctx.diagnostic(diagnostic);
            } else {
                // Collect separators between consecutive sortable items.
                let mut separators: Vec<&str> = Vec::with_capacity(sortable_idx.len());
                for i in 0..sortable_idx.len() {
                    if i + 1 < sortable_idx.len() {
                        let curr_ext = cg[sortable_idx[i]].extended_end;
                        let next_start = attr_span(group[sortable_idx[i + 1]]).start;
                        separators.push(ctx.source_range(Span::new(curr_ext, next_start)));
                    } else {
                        separators.push("");
                    }
                }

                // Build the sorted replacement text.
                let mut sorted_text = String::new();
                for (pos, &orig_pos) in sorted_order.iter().enumerate() {
                    let idx = sortable_idx[orig_pos];
                    let start = attr_span(group[idx]).start;
                    let end = cg[idx].extended_end;
                    sorted_text.push_str(ctx.source_range(Span::new(start, end)));
                    if pos + 1 < sorted_order.len() {
                        let sep = if pos < separators.len() && !separators[pos].is_empty() {
                            separators[pos]
                        } else {
                            " "
                        };
                        sorted_text.push_str(sep);
                    }
                }

                ctx.diagnostic_with_fix(diagnostic, |fixer| fixer.replace(group_span, sorted_text));
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

/// Get the span of a JSXAttributeItem.
fn attr_span(attr: &JSXAttributeItem) -> Span {
    match attr {
        JSXAttributeItem::Attribute(a) => a.span,
        JSXAttributeItem::SpreadAttribute(s) => s.span,
    }
}

/// Collect groups of JSXAttributeItems separated by spread attributes.
fn collect_groups<'a, 'b>(
    attributes: &'b [JSXAttributeItem<'a>],
) -> Vec<Vec<&'b JSXAttributeItem<'a>>> {
    let mut groups: Vec<Vec<&JSXAttributeItem<'a>>> = vec![vec![]];
    for attr in attributes {
        match attr {
            JSXAttributeItem::SpreadAttribute(_) => {
                groups.push(vec![]);
            }
            JSXAttributeItem::Attribute(_) => {
                groups.last_mut().unwrap().push(attr);
            }
        }
    }
    groups
}

/// Information about a single JSX prop used for sorting.
#[derive(Clone)]
struct PropInfo {
    name: String,
    /// The sort key (potentially lowercased).
    sort_key: String,
    group_rank: u8,
    /// Index in the `sortFirst` list, if this prop is in that list.
    sort_first_index: Option<usize>,
}

/// Classify a prop into its group rank and extract its name.
fn classify_prop(
    attr: &JSXAttributeItem,
    config: &JsxSortPropsConfig,
    reserved_list: &[&str],
    ctx: &LintContext,
) -> PropInfo {
    let JSXAttributeItem::Attribute(jsx_attr) = attr else {
        unreachable!("spread attributes are filtered out before calling classify_prop");
    };

    let name = match &jsx_attr.name {
        JSXAttributeName::Identifier(ident) => ident.name.to_string(),
        JSXAttributeName::NamespacedName(ns) => {
            format!("{}:{}", ns.namespace.name, ns.name.name)
        }
    };

    let is_callback = is_callback_prop(&name);
    let is_shorthand = jsx_attr.value.is_none();
    let is_multiline = is_multiline_prop(attr_span(attr), ctx);
    let is_reserved = !reserved_list.is_empty() && reserved_list.contains(&name.as_str());

    // Check if this prop is in the sortFirst list.
    let sort_first_index = if config.sort_first.is_empty() {
        None
    } else {
        config.sort_first.iter().position(|s| {
            if config.ignore_case { s.eq_ignore_ascii_case(&name) } else { s == &name }
        })
    };

    // Assign group rank based on config.
    // Lower rank = appears first. Different "last" groups get distinct ranks
    // so they sort in the correct relative order:
    //   reserved(0) < shorthandFirst(1) < multilineFirst(2) < regular(3)
    //   < multilineLast(4) < shorthandLast(5) < callbacksLast(6)
    // Note: sortFirst takes priority over all group ranks and is handled
    // separately in compare_props via sort_first_index.
    let group_rank = if is_reserved {
        0
    } else if config.shorthand_first && is_shorthand && !is_callback {
        1
    } else if matches!(config.multiline, MultilineOption::First) && is_multiline {
        2
    } else if config.callbacks_last && is_callback {
        6
    } else if config.shorthand_last && is_shorthand {
        5
    } else if matches!(config.multiline, MultilineOption::Last) && is_multiline {
        4
    } else {
        3 // regular
    };

    let sort_key =
        if config.ignore_case { name.cow_to_ascii_lowercase().into_owned() } else { name.clone() };

    PropInfo { name, sort_key, group_rank, sort_first_index }
}

/// Check if a prop name is a callback (starts with "on" followed by uppercase).
fn is_callback_prop(name: &str) -> bool {
    if let Some(rest) = name.strip_prefix("on") {
        rest.starts_with(|c: char| c.is_ascii_uppercase())
    } else {
        false
    }
}

/// Check if an attribute spans multiple lines.
fn is_multiline_prop(span: Span, ctx: &LintContext) -> bool {
    ctx.source_range(span).contains('\n')
}

/// Compare two props for sorting.
fn compare_props(
    a: &PropInfo,
    b: &PropInfo,
    config: &JsxSortPropsConfig,
    collator: Option<&CollatorBorrowed<'_>>,
) -> Ordering {
    // sortFirst takes highest priority: props in sortFirst come before all others,
    // and within sortFirst they maintain the order specified in the config array.
    match (a.sort_first_index, b.sort_first_index) {
        (Some(ai), Some(bi)) => {
            let cmp = ai.cmp(&bi);
            if cmp != Ordering::Equal {
                return cmp;
            }
            // Same sortFirst index (duplicate prop names) — treat as equal.
            return Ordering::Equal;
        }
        (Some(_), None) => return Ordering::Less,
        (None, Some(_)) => return Ordering::Greater,
        (None, None) => {}
    }

    // Then compare by group rank.
    let rank_cmp = a.group_rank.cmp(&b.group_rank);
    if rank_cmp != Ordering::Equal {
        return rank_cmp;
    }

    // Within the same group, sort alphabetically (unless disabled).
    if config.no_sort_alphabetically {
        return Ordering::Equal;
    }

    // When an explicit locale is configured, use ICU collation.
    // The collator already handles case sensitivity via Strength settings.
    // When no collator (locale "auto"/empty), use byte-order on sort_key
    // (which is already lowercased when ignore_case is true).
    if let Some(collator) = collator {
        collator.compare(&a.name, &b.name)
    } else {
        a.sort_key.cmp(&b.sort_key)
    }
}

/// Create an ICU collator for the given locale string.
/// Returns `None` for "auto" or empty locale (use byte-order comparison).
/// Returns `Some(collator)` for explicit locales like "sk-SK".
fn create_collator(locale_str: &str, ignore_case: bool) -> Option<CollatorBorrowed<'static>> {
    if locale_str.is_empty() || locale_str == "auto" {
        return None;
    }

    let locale: Locale = locale_str.parse().ok()?;

    let mut options = CollatorOptions::default();
    options.strength = Some(if ignore_case {
        // Primary strength ignores case and accents, matching ESLint's
        // behavior of lowercasing + localeCompare with ignoreCase.
        Strength::Secondary
    } else {
        // Tertiary strength considers case differences (default).
        Strength::Tertiary
    });

    Collator::try_new(locale.into(), options).ok()
}

/// Check if the JSX element is a DOM component (starts with lowercase letter).
/// In JSX, DOM components like `<div>`, `<span>` use `JSXElementName::Identifier`,
/// while custom components like `<App>` use `JSXElementName::IdentifierReference`.
fn is_dom_component(jsx_opening_elem: &JSXOpeningElement) -> bool {
    matches!(jsx_opening_elem.name, JSXElementName::Identifier(_))
}

// ---------------------------------------------------------------------------
// Comment-grouping support (ESLint-compatible)
// ---------------------------------------------------------------------------

/// Tracks how an attribute's text range is extended to include trailing
/// comments and potentially the next consumed attribute.
struct CommentGrouping {
    /// End of the extended source range (may exceed `attr.span.end` when
    /// trailing comments or a consumed next attribute are included).
    extended_end: u32,
    /// When `true` the attribute sorts to the **end** of its group.
    /// This is set when a block comment or next-line comment bridges to the
    /// next attribute (ESLint's `hasComment` flag).
    has_comment: bool,
    /// When `true` this attribute was absorbed into the previous attribute's
    /// extended range and should **not** appear as a standalone sortable item.
    consumed: bool,
}

impl CommentGrouping {
    fn default_for(end: u32) -> Self {
        Self { extended_end: end, has_comment: false, consumed: false }
    }
}

/// Count the number of newline characters (`\n`) between two byte offsets.
fn count_newlines(source: &str, from: u32, to: u32) -> usize {
    source[from as usize..to as usize].bytes().filter(|&b| b == b'\n').count()
}

/// For a consumed next attribute, check whether it has a single same-line
/// trailing comment and, if so, extend the range to include that comment.
fn extended_end_with_next_trailing(
    group: &[&JSXAttributeItem],
    next_idx: usize,
    elem_end: u32,
    next_span: Span,
    ctx: &LintContext,
) -> u32 {
    let source = ctx.source_text();
    let after_next_end = group.get(next_idx + 1).map_or(elem_end, |na| attr_span(na).start);
    let next_comments: Vec<_> = ctx.comments_range(next_span.end..after_next_end).collect();

    if next_comments.len() == 1 {
        let nc = next_comments[0];
        let nc_newlines = count_newlines(source, next_span.start, nc.span.start);
        if nc_newlines == 0 {
            // Same-line trailing comment on the consumed attribute.
            return nc.span.end;
        }
    }
    next_span.end
}

/// Build the comment-grouping map for every attribute in `group`.
///
/// The algorithm mirrors ESLint's `getGroupsOfSortableAttributes` logic:
/// it inspects trailing comments after each attribute and decides whether
/// to consume the next attribute into the current one's extended range.
fn compute_comment_grouping(
    group: &[&JSXAttributeItem],
    elem_end: u32,
    ctx: &LintContext,
) -> Vec<CommentGrouping> {
    let source = ctx.source_text();
    let mut result = Vec::with_capacity(group.len());
    let mut i = 0;

    while i < group.len() {
        let span = attr_span(group[i]);
        let next_attr = group.get(i + 1);
        let search_end = next_attr.map_or(elem_end, |na| attr_span(na).start);

        let comments: Vec<_> = ctx.comments_range(span.end..search_end).collect();

        if comments.is_empty() {
            // No trailing comments — plain attribute.
            result.push(CommentGrouping::default_for(span.end));
            i += 1;
            continue;
        }

        let first = comments[0];
        // Count newlines from the attribute START to match ESLint's
        // `attribute.loc.start.line` comparison.
        let first_nl = count_newlines(source, span.start, first.span.start);
        let first_same_line = first_nl == 0;
        let first_next_line = first_nl == 1;

        if comments.len() == 1 {
            if first_same_line && first.is_block() {
                // Same-line block comment `/* ... */`.
                if let Some(next) = next_attr {
                    let next_span = attr_span(next);
                    let ext =
                        extended_end_with_next_trailing(group, i + 1, elem_end, next_span, ctx);
                    result.push(CommentGrouping {
                        extended_end: ext,
                        has_comment: true,
                        consumed: false,
                    });
                    result.push(CommentGrouping {
                        extended_end: ext,
                        has_comment: false,
                        consumed: true,
                    });
                    i += 2;
                } else {
                    // Block comment after the last attribute.
                    result.push(CommentGrouping {
                        extended_end: first.span.end,
                        has_comment: true,
                        consumed: false,
                    });
                    i += 1;
                }
            } else if first_same_line {
                // Same-line line comment `// ...` — travels with the
                // attribute but does NOT force sort-to-end.
                result.push(CommentGrouping {
                    extended_end: first.span.end,
                    has_comment: false,
                    consumed: false,
                });
                i += 1;
            } else if first_next_line && next_attr.is_some() {
                // Comment on the next line, with a following attribute.
                let next = next_attr.unwrap();
                let next_span = attr_span(next);
                let ext = extended_end_with_next_trailing(group, i + 1, elem_end, next_span, ctx);
                result.push(CommentGrouping {
                    extended_end: ext,
                    has_comment: true,
                    consumed: false,
                });
                result.push(CommentGrouping {
                    extended_end: ext,
                    has_comment: false,
                    consumed: true,
                });
                i += 2;
            } else {
                // Comment is ≥2 lines away, or next-line with no following
                // attribute — do not extend.
                result.push(CommentGrouping::default_for(span.end));
                i += 1;
            }
        } else {
            // Two or more comments after this attribute.
            let second_nl = count_newlines(source, span.start, comments[1].span.start);
            let second_next_line = second_nl == 1;

            if second_next_line && next_attr.is_some() {
                let next = next_attr.unwrap();
                let next_span = attr_span(next);
                let ext = extended_end_with_next_trailing(group, i + 1, elem_end, next_span, ctx);
                result.push(CommentGrouping {
                    extended_end: ext,
                    has_comment: true,
                    consumed: false,
                });
                result.push(CommentGrouping {
                    extended_end: ext,
                    has_comment: false,
                    consumed: true,
                });
                i += 2;
            } else {
                // Default for multiple comments that don't match the
                // next-line pattern — no extension.
                result.push(CommentGrouping::default_for(span.end));
                i += 1;
            }
        }
    }

    result
}

/// Find the first violation and return the appropriate diagnostic function.
fn find_first_violation(
    original: &[PropInfo],
    sorted: &[PropInfo],
    config: &JsxSortPropsConfig,
) -> fn(Span) -> OxcDiagnostic {
    for (orig, sort) in original.iter().zip(sorted.iter()) {
        if orig.name == sort.name && orig.group_rank == sort.group_rank {
            continue;
        }

        // Determine what kind of violation this is.

        // Check sortFirst violations: a non-sortFirst prop appears where a
        // sortFirst prop should be, or two sortFirst props are in wrong order.
        if sort.sort_first_index.is_some() && orig.sort_first_index != sort.sort_first_index {
            return sort_first_diagnostic;
        }

        // Check if a non-reserved prop appears where a reserved prop should be.
        if sort.group_rank == 0 && orig.group_rank != 0 {
            return reserved_first_diagnostic;
        }

        // Check shorthand-first violation.
        if sort.group_rank == 1 && orig.group_rank != 1 {
            return shorthand_first_diagnostic;
        }

        // Check multiline-first violation.
        if sort.group_rank == 2 && orig.group_rank != 2 {
            return multiline_first_diagnostic;
        }

        // Check callbacks-last violation: a callback is where a non-callback should be.
        if orig.group_rank == 6 && sort.group_rank != 6 {
            return callbacks_last_diagnostic;
        }

        // Check multiline-last violation.
        if matches!(config.multiline, MultilineOption::Last)
            && orig.group_rank == 4
            && sort.group_rank != 4
        {
            return multiline_last_diagnostic;
        }

        // Check shorthand-last violation.
        if config.shorthand_last && orig.group_rank == 5 && sort.group_rank != 5 {
            return shorthand_last_diagnostic;
        }

        // Default: alphabetical order violation.
        return alphabetical_diagnostic;
    }

    // Fallback.
    alphabetical_diagnostic
}

#[test]
fn test() {
    use crate::tester::Tester;

    let ignore_case_args = serde_json::json!([{ "ignoreCase": true }]);
    let callbacks_last_args = serde_json::json!([{ "callbacksLast": true }]);
    let ignore_case_and_callback_last_args =
        serde_json::json!([{ "ignoreCase": true, "callbacksLast": true }]);
    let shorthand_first_args = serde_json::json!([{ "shorthandFirst": true }]);
    let shorthand_last_args = serde_json::json!([{ "shorthandLast": true }]);
    let shorthand_and_callback_last_args =
        serde_json::json!([{ "shorthandLast": true, "callbacksLast": true }]);
    let multiline_first_args = serde_json::json!([{ "multiline": "first" }]);
    let multiline_last_args = serde_json::json!([{ "multiline": "last" }]);
    let multiline_and_shorthand_first_args =
        serde_json::json!([{ "multiline": "first", "shorthandFirst": true }]);
    let multiline_and_shorthand_and_callback_last_args =
        serde_json::json!([{ "multiline": "last", "shorthandLast": true, "callbacksLast": true }]);
    let no_sort_alphabetically_args = serde_json::json!([{ "noSortAlphabetically": true }]);
    let sort_alphabetically_args = serde_json::json!([{ "noSortAlphabetically": false }]);
    let reserved_first_as_boolean_args = serde_json::json!([{ "reservedFirst": true }]);
    let reserved_first_as_array_args =
        serde_json::json!([{ "reservedFirst": ["children", "dangerouslySetInnerHTML", "key"] }]);
    let reserved_first_with_no_sort_alphabetically_args =
        serde_json::json!([{ "reservedFirst": true, "noSortAlphabetically": true }]);
    let reserved_first_as_empty_array_args = serde_json::json!([{ "reservedFirst": [] }]);
    let reserved_first_as_invalid_array_args =
        serde_json::json!([{ "reservedFirst": ["notReserved"] }]);
    let reserved_first_and_callbacks_last_args =
        serde_json::json!([{ "reservedFirst": true, "callbacksLast": true }]);
    let reserved_first_with_shorthand_last =
        serde_json::json!([{ "reservedFirst": true, "shorthandLast": true }]);
    let sort_first_args = serde_json::json!([{ "sortFirst": ["className"] }]);
    let sort_first_multiple_args = serde_json::json!([{ "sortFirst": ["className", "id"] }]);
    let sort_first_with_ignore_case_args =
        serde_json::json!([{ "sortFirst": ["className"], "ignoreCase": true }]);
    let sort_first_with_reserved_first_args =
        serde_json::json!([{ "sortFirst": ["className"], "reservedFirst": true }]);
    let sort_first_with_shorthand_first_args =
        serde_json::json!([{ "sortFirst": ["className"], "shorthandFirst": true }]);
    let sort_first_with_callbacks_last_args =
        serde_json::json!([{ "sortFirst": ["className"], "callbacksLast": true }]);
    let sort_first_with_multiline_first_args =
        serde_json::json!([{ "sortFirst": ["className"], "multiline": "first" }]);

    let pass = vec![
        ("<App />;", None),
        ("<App {...this.props} />;", None),
        ("<App a b c />;", None),
        ("<App {...this.props} a b c />;", None),
        ("<App c {...this.props} a b />;", None),
        (r#"<App a="c" b="b" c="a" />;"#, None),
        (r#"<App {...this.props} a="c" b="b" c="a" />;"#, None),
        (r#"<App c="a" {...this.props} a="c" b="b" />;"#, None),
        ("<App A a />;", None),
        ("<App aB aa/>;", None),
        ("<App aA aB />;", None),
        ("<App aB aaa />;", None),
        ("<App a aB aa />;", None),
        (r#"<App Number="2" name="John" />;"#, None),
        ("<App a A />;", Some(ignore_case_args.clone())),
        ("<App aa aB />;", Some(ignore_case_args.clone())),
        ("<App a B c />;", Some(ignore_case_args.clone())),
        ("<App A b C />;", Some(ignore_case_args.clone())),
        (r#"<App name="John" Number="2" />;"#, Some(ignore_case_args.clone())),
        ("<App a z onBar onFoo />;", Some(callbacks_last_args.clone())),
        ("<App z onBar onFoo />;", Some(ignore_case_and_callback_last_args)),
        (r#"<App a b="b" />;"#, Some(shorthand_first_args.clone())),
        (r#"<App z a="a" />;"#, Some(shorthand_first_args.clone())),
        (r#"<App x y z a="a" b="b" />;"#, Some(shorthand_first_args.clone())),
        (r#"<App a="a" b="b" x y z />;"#, Some(shorthand_last_args.clone())),
        (r#"<App a="a" b="b" x y z onBar onFoo />;"#, Some(shorthand_and_callback_last_args)),
        (
            "
                    <App
                      a={{
                        aA: 1,
                      }}
                      b
                    />
                  ",
            Some(multiline_first_args.clone()),
        ),
        (
            "
                    <App
                      a={{
                        aA: 1,
                      }}
                      b={[
                        1,
                      ]}
                      c
                      d
                    />
                  ",
            Some(multiline_first_args.clone()),
        ),
        (
            r#"
                    <App
                      a
                      b
                      c={{
                        cC: 1,
                      }}
                      d={[
                        1,
                      ]}
                      e="1"
                    />
                  "#,
            Some(multiline_and_shorthand_first_args.clone()),
        ),
        (
            "
                    <App
                      a
                      b={{
                        bB: 1,
                      }}
                    />
                  ",
            Some(multiline_last_args.clone()),
        ),
        (
            r#"
                    <App
                      a
                      b
                      c="1"
                      d={{
                        dD: 1,
                      }}
                      e={[
                        1,
                      ]}
                    />
                  "#,
            Some(multiline_last_args.clone()),
        ),
        (
            r#"
                    <App
                      a={1}
                      b="1"
                      c={{
                        cC: 1,
                      }}
                      d={() => (
                        1
                      )}
                      e
                      f
                      onClick={() => ({
                        gG: 1,
                      })}
                    />
                  "#,
            Some(multiline_and_shorthand_and_callback_last_args.clone()),
        ),
        ("<App a b />;", Some(no_sort_alphabetically_args.clone())),
        ("<App b a />;", Some(no_sort_alphabetically_args)),
        (
            r#"<App children={<App />} key={0} ref="r" a b c />"#,
            Some(reserved_first_as_boolean_args.clone()),
        ),
        // On non-DOM components (like <App>), dangerouslySetInnerHTML is NOT reserved,
        // so it sorts alphabetically after non-reserved props. This is correct.
        (
            r#"<App children={<App />} key={0} ref="r" a b c dangerouslySetInnerHTML={{__html: "EPR"}} />"#,
            Some(reserved_first_as_boolean_args.clone()),
        ),
        (
            r#"<App children={<App />} key={0} a ref="r" />"#,
            Some(reserved_first_as_array_args.clone()),
        ),
        (
            r#"<App children={<App />} key={0} a dangerouslySetInnerHTML={{__html: "EPR"}} ref="r" />"#,
            Some(reserved_first_as_array_args.clone()),
        ),
        (
            r#"<App ref="r" key={0} children={<App />} b a c />"#,
            Some(reserved_first_with_no_sort_alphabetically_args.clone()),
        ),
        (
            r#"<div ref="r" dangerouslySetInnerHTML={{__html: "EPR"}} key={0} children={<App />} b a c />"#,
            Some(reserved_first_with_no_sort_alphabetically_args.clone()),
        ),
        (r#"<App key="key" c="c" b />"#, Some(reserved_first_with_shorthand_last.clone())),
        (
            "
                    <RawFileField
                      onChange={handleChange}
                      onFileRemove={asMedia ? null : handleRemove}
                      {...props}
                    />
                  ",
            None,
        ),
        // Locale-aware sorting: explicit locale uses ICU collation.
        // In Slovak, "ch" is a digraph sorting after "h", so onChange (with "Ch")
        // may sort differently than in byte-order. Use props without such digraphs.
        (
            "
                    <RawFileField
                      disabled={isDisabled}
                      name={fieldName}
                      {...props}
                    />
                  ",
            Some(serde_json::json!([{ "locale": "sk-SK" }])),
        ),
        // Slovak locale: č sorts between c and d, ä sorts after a but before b.
        // In byte-order (no locale), ä and č would sort after all ASCII letters.
        // With sk-SK locale, this order is correct.
        (r"<App a ä b č d />", Some(serde_json::json!([{ "locale": "sk" }]))),
        // "auto" locale falls back to byte-order comparison (matching ESLint default).
        (r"<App a b c />", Some(serde_json::json!([{ "locale": "auto" }]))),
        // Locale with ignoreCase: case-insensitive collation.
        (r"<App a B c />", Some(serde_json::json!([{ "locale": "en", "ignoreCase": true }]))),
        // Spread boundary tests: each spread resets the sorting group.
        // Props within each group must be sorted, but groups are independent.
        ("<App b {...rest} a c />;", None),
        ("<App a b {...rest} c d />;", None),
        ("<App a {...x} b {...y} c />;", None),
        ("<App a b {...x} c d {...y} e f />;", None),
        // Single prop in a group is always valid.
        ("<App z {...rest} a />;", None),
        ("<App a {...x} z {...y} a />;", None),
        // Multiple consecutive spreads create empty groups (valid).
        ("<App a {...x} {...y} b />;", None),
        ("<App c {...x} {...y} {...z} a />;", None),
        // Reserved prop `key` listed before regular prop `as` on non-DOM component.
        (r#"<Box key={index} as="span" />"#, Some(reserved_first_as_boolean_args.clone())),
        // sortFirst: single prop listed first.
        (r#"<App className="test" name="John" />"#, Some(sort_first_args.clone())),
        // sortFirst: multiple props listed first in order.
        (
            r#"<App className="test" id="test" name="John" />"#,
            Some(sort_first_multiple_args.clone()),
        ),
        // sortFirst: only sortFirst props.
        (r#"<App className="test" id="test" />"#, Some(sort_first_multiple_args.clone())),
        // sortFirst: followed by alphabetical regular props.
        (r#"<App className="test" a b c />"#, Some(sort_first_args.clone())),
        // sortFirst: multiple, followed by alphabetical regular props.
        (r#"<App className="test" id="test" a b c />"#, Some(sort_first_multiple_args.clone())),
        // sortFirst + reservedFirst: sortFirst before reserved before regular.
        (
            r#"<App className="test" key={0} name="John" />"#,
            Some(sort_first_with_reserved_first_args.clone()),
        ),
        // sortFirst + shorthandFirst: sortFirst before shorthand before regular.
        (
            r#"<App className="test" a name="John" />"#,
            Some(sort_first_with_shorthand_first_args.clone()),
        ),
        // sortFirst + callbacksLast: sortFirst before regular, callbacks last.
        (
            r#"<App className="test" name="John" onClick={handleClick} />"#,
            Some(sort_first_with_callbacks_last_args.clone()),
        ),
        // sortFirst + multiline first.
        (
            r#"
                    <App
                      className="test"
                      data={{
                        test: 1,
                      }}
                      name="John"
                    />
                  "#,
            Some(sort_first_with_multiline_first_args.clone()),
        ),
        // sortFirst + ignoreCase: case-insensitive matching of sortFirst list.
        (r#"<App classname="test" a="test2" />"#, Some(sort_first_with_ignore_case_args.clone())),
    ];

    let fail = vec![
        ("<App b a />;", None),
        ("<App aB a />;", None),
        (
            r#"<App fistName="John" tel={5555555} name="John Smith" lastName="Smith" Number="2" />;"#,
            None,
        ),
        ("<App aa aB />;", None),
        ("<App aB aA />;", None),
        ("<App aaB aA />;", None),
        ("<App aaB aaa aA a />;", None),
        ("<App {...this.props} b a />;", None),
        ("<App c {...this.props} b a />;", None),
        (
            r#"<App fistName="John" tel={5555555} name="John Smith" lastName="Smith" Number="2" />;"#,
            Some(ignore_case_args.clone()),
        ),
        ("<App B a />;", Some(ignore_case_args.clone())),
        ("<App B A c />;", Some(ignore_case_args.clone())),
        (r#"<App c="a" a="c" b="b" />;"#, None),
        (r#"<App {...this.props} c="a" a="c" b="b" />;"#, None),
        (r#"<App d="d" b="b" {...this.props} c="a" a="c" />;"#, None),
        (
            "
                    <App
                      a={true}
                      z
                      r
                      _onClick={function(){}}
                      onHandle={function(){}}
                      {...this.props}
                      b={false}
                      {...otherProps}
                    >
                      {test}
                    </App>
                  ",
            None,
        ),
        ("<App b={2} c={3} d={4} e={5} f={6} g={7} h={8} i={9} j={10} k={11} a={1} />", None),
        (
            "
                    <List
                      className={className}
                      onStageAnswer={onStageAnswer}
                      onCommitAnswer={onCommitAnswer}
                      isFocused={isFocused}
                      direction={direction}
                      allowMultipleSelection={allowMultipleSelection}
                      measureLongestChildNode={measureLongestChildNode}
                      layoutItemsSize={layoutItemsSize}
                      handleAppScroll={handleAppScroll}
                      isActive={isActive}
                      resetSelection={resetSelection}
                      onKeyboardChoiceHovered={onKeyboardChoiceHovered}
                      keyboardShortcutType
                    />
                  ",
            None,
        ),
        (
            "
                    <CreateNewJob
                      closed={false}
                      flagOptions={flagOptions}
                      jobHeight={300}
                      jobWidth={200}
                      campaign='Some Campaign name'
                      campaignStart={moment('2018-07-28 00:00:00')}
                      campaignFinish={moment('2018-09-01 00:00:00')}
                      jobNumber={'Job Number can be a String'}
                      jobTemplateOptions={jobTemplateOptions}
                      numberOfPages={30}
                      onChange={onChange}
                      onClose={onClose}
                      spreadSheetTemplateOptions={spreadSheetTemplateOptions}
                      stateMachineOptions={stateMachineOptions}
                      workflowTemplateOptions={workflowTemplateOptions}
                      workflowTemplateSteps={workflowTemplateSteps}
                      description='Some description for this job'
            
                      jobTemplate='1'
                      stateMachine='1'
                      flag='1'
                      spreadSheetTemplate='1'
                      workflowTemplate='1'
                      validation={validation}
                      onSubmit={onSubmit}
                    />
                  ",
            None,
        ),
        (r#"<App key="key" b c="c" />"#, Some(reserved_first_with_shorthand_last.clone())),
        (
            r#"<App ref="ref" key="key" isShorthand veryLastAttribute="yes" />"#,
            Some(reserved_first_with_shorthand_last.clone()),
        ),
        ("<App a z onFoo onBar />;", Some(callbacks_last_args.clone())),
        ("<App a onBar onFoo z />;", Some(callbacks_last_args.clone())),
        (r#"<App a="a" b />;"#, Some(shorthand_first_args.clone())),
        (r#"<App z x a="a" />;"#, Some(shorthand_first_args.clone())),
        (r#"<App b a="a" />;"#, Some(shorthand_last_args.clone())),
        (r#"<App a="a" onBar onFoo z x />;"#, Some(shorthand_last_args.clone())),
        ("<App b a />;", Some(sort_alphabetically_args.clone())),
        ("<App a key={1} />", Some(reserved_first_as_boolean_args.clone())),
        (
            r#"<div a dangerouslySetInnerHTML={{__html: "EPR"}} />"#,
            Some(reserved_first_as_boolean_args.clone()),
        ),
        (r#"<App ref="r" key={2} b />"#, Some(reserved_first_as_boolean_args.clone())),
        ("<App key={2} b a />", Some(reserved_first_as_boolean_args.clone())),
        ("<App b a />", Some(reserved_first_as_boolean_args.clone())),
        (
            r#"<App dangerouslySetInnerHTML={{__html: "EPR"}} e key={2} b />"#,
            Some(reserved_first_as_boolean_args.clone()),
        ),
        ("<App key={3} children={<App />} />", Some(reserved_first_as_array_args.clone())),
        (r#"<App z ref="r" />"#, Some(reserved_first_with_no_sort_alphabetically_args.clone())),
        // Empty reservedFirst array → report listIsEmpty error on every prop
        ("<App key={4} />", Some(reserved_first_as_empty_array_args)),
        // Invalid reservedFirst array → report noUnreservedProps error on every prop
        ("<App key={5} />", Some(reserved_first_as_invalid_array_args)),
        ("<App onBar z />;", Some(reserved_first_and_callbacks_last_args.clone())),
        // Reserved prop `key` after regular prop `as` on non-DOM component.
        (r#"<Box as="span" key={index} />"#, Some(reserved_first_as_boolean_args.clone())),
        (
            "
                    <App
                      a
                      b={{
                        bB: 1,
                      }}
                    />
                  ",
            Some(multiline_first_args.clone()),
        ),
        (
            "
                    <App
                      a={1}
                      b={{
                        bB: 1,
                      }}
                      c
                    />
                  ",
            Some(multiline_and_shorthand_first_args.clone()),
        ),
        (
            "
                    <App
                      a={{
                        aA: 1,
                      }}
                      b
                    />
                  ",
            Some(multiline_last_args.clone()),
        ),
        (
            r#"
                    <App
                      a={{
                        aA: 1,
                      }}
                      b
                      inline={1}
                      onClick={() => ({
                        c: 1
                      })}
                      d="dD"
                      e={() => ({
                        eE: 1
                      })}
                      f
                    />
                  "#,
            Some(multiline_and_shorthand_and_callback_last_args.clone()),
        ),
        (
            r#"
                    <Typography
                      float
                      className={classNames(classes.inputWidth, {
                        [classes.noBorder]: isActive === "values",
                      })}
                      disabled={isDisabled}
                      initialValue={computePercentage(number, count)}
                      InputProps={{
                        ...customInputProps,
                      }}
                      key={index}
                      isRequired
                      {...sharedTypographyProps}
                      ref={textRef}
                      min="0"
                      name="fieldName"
                      placeholder={getTranslation("field")}
                      onValidate={validate}
                      inputProps={{
                        className: inputClassName,
                      }}
                      outlined
                      {...rest}
                    />
                  "#,
            Some(
                serde_json::json!([ { "multiline": "last", "shorthandFirst": true, "callbacksLast": true, "reservedFirst": true, "ignoreCase": true, }, ]),
            ),
        ),
        (
            r#"
                    <Page
                      // Pass all the props to the Page component.
                      {...props}
                      // Use the platform specific props from the doc.ts file.
                      {...TemplatePageProps[platform]}
                      // Use the getSubTitle helper function to get the page header subtitle from the active platform.
                      subTitle={getSubTitle(platform)}
                      // You can define custom sections using the `otherSections` prop.
                      // Here it is using a method that takes the platform as an argument to return the correct array of section props.
                      otherSections={_otherSections(platform) as IPageSectionProps[]}
            
                      // You can hide the side rail by setting `showSideRail` to false.
                      // showSideRail={false}
            
                      // You can pass a custom className to the page wrapper if needed.
                      // className="customPageClassName"
                    />
                  "#,
            None,
        ),
        // Spread boundary tests: unsorted props within a group after spread.
        ("<App b {...rest} c a />;", None),
        // Unsorted in both groups separated by spread.
        ("<App d c {...rest} b a />;", None),
        // Multiple spreads with unsorted group in the middle.
        ("<App a {...x} d c {...y} e />;", None),
        // Multiple spreads with unsorted groups in all segments.
        ("<App b a {...x} d c {...y} f e />;", None),
        // Unsorted only in the last group after multiple spreads.
        ("<App a {...x} b {...y} d c />;", None),
        // Locale-aware fail tests:
        // In Slovak, ä sorts between a and b, but d comes before ä in byte-order
        // only when ä is after all ASCII. Here ä should come before b in sk locale,
        // so "b ä" is wrong order under sk collation.
        (r"<App b ä />", Some(serde_json::json!([{ "locale": "sk" }]))),
        // In Slovak, č sorts between c and d. So "d č" is wrong under sk collation.
        (r"<App d č />", Some(serde_json::json!([{ "locale": "sk" }]))),
        // sortFirst: className should be listed first.
        (r#"<App name="John" className="test" />"#, Some(sort_first_args.clone())),
        // sortFirst: multiple — className before id.
        (
            r#"<App id="test" className="test" name="John" />"#,
            Some(sort_first_multiple_args.clone()),
        ),
        // sortFirst: className should be before regular prop a.
        (r#"<App a className="test" b />"#, Some(sort_first_args.clone())),
        // sortFirst + reservedFirst: sortFirst takes priority over reserved.
        (
            r#"<App key={0} className="test" name="John" />"#,
            Some(sort_first_with_reserved_first_args.clone()),
        ),
        // sortFirst + shorthandFirst: sortFirst takes priority.
        (
            r#"<App a className="test" name="John" />"#,
            Some(sort_first_with_shorthand_first_args.clone()),
        ),
        // sortFirst + callbacksLast: className should be first.
        (
            r#"<App name="John" onClick={handleClick} className="test" />"#,
            Some(sort_first_with_callbacks_last_args.clone()),
        ),
        // sortFirst + multiline first.
        (
            r#"
                    <App
                      name="John"
                      className="test"
                      data={{
                        test: 1,
                      }}
                    />
                  "#,
            Some(sort_first_with_multiline_first_args.clone()),
        ),
        // sortFirst + ignoreCase: classname matches className case-insensitively.
        (r#"<App name="John" classname="test" />"#, Some(sort_first_with_ignore_case_args.clone())),
        // sortFirst: regular props after sortFirst should still be alphabetical.
        (
            r#"<App className="test" id="test" tel={5555555} name="John" />"#,
            Some(sort_first_multiple_args.clone()),
        ),
        // sortFirst: duplicate prop name with wrong order.
        (
            r#"<App id="test" className="test" id="test2" />"#,
            Some(sort_first_multiple_args.clone()),
        ),
        // ── Comment-handling fail tests ──────────────────────────
        // Test 1: same-line line comments + standalone comment between attrs.
        (
            r#"<foo
  m={0}
  n={0} // this is n
  o={0}
  c={0} // this is c
  // fofof
  f={0} // this is f
  a={0}
  b={0}
  d={0}
/>"#,
            None,
        ),
        // Test 2: all same-line line comments (no grouping needed).
        (
            r#"<foo
  m={0}
  n={0} // this is n
  o={0}
  c={0} // this is c
  f={0} // this is f
  e={0}
  a={0}
  b={0}
  d={0}
/>"#,
            None,
        ),
        // Test 3: mixed same-line and next-line comments consuming attrs.
        (
            r#"<foo
  a1={0}
  g={0}
  d={0} // comment for d
  // comment for d and aa
  aa={0}
  c={0} // comment for c
  // comment for c and e
  e={1}
  ab={1} // comment for ab
  f={0}
/>"#,
            None,
        ),
        // Test 4: next-line comment consuming following attr.
        (
            r#"<foo
  a1={0}
  ab={1}
  // comment for ab and f
  f={0}
  g={0}
  c={0} // comment for c
  // comment for c and e
  e={1}
  d={0}
  aa={1} // comment for aa
/>"#,
            None,
        ),
        // Test 5: inline block comment /* */ between attrs on same line.
        ("<foo a={0} b={1} /* comment for b and ab */ ab={1} aa={0} />", None),
        // Test 6: block comment at end of attribute list (callbacksLast).
        (
            r#"<ReactJson src={rowResult} name="data" collapsed={4} collapseStringsAfterLength={60} onEdit={onEdit} /* onDelete={onEdit} */ />"#,
            Some(callbacks_last_args.clone()),
        ),
    ];

    let fix = vec![
        ("<App b a />;", "<App a b />;", None),
        ("<App aB a />;", "<App a aB />;", None),
        (
            r#"<App fistName="John" tel={5555555} name="John Smith" lastName="Smith" Number="2" />;"#,
            r#"<App Number="2" fistName="John" lastName="Smith" name="John Smith" tel={5555555} />;"#,
            None,
        ),
        ("<App aa aB />;", "<App aB aa />;", None),
        ("<App aB aA />;", "<App aA aB />;", None),
        ("<App aaB aA />;", "<App aA aaB />;", None),
        ("<App aaB aaa aA a />;", "<App a aA aaB aaa />;", None),
        ("<App {...this.props} b a />;", "<App {...this.props} a b />;", None),
        ("<App c {...this.props} b a />;", "<App c {...this.props} a b />;", None),
        (
            r#"<App fistName="John" tel={5555555} name="John Smith" lastName="Smith" Number="2" />;"#,
            r#"<App fistName="John" lastName="Smith" name="John Smith" Number="2" tel={5555555} />;"#,
            Some(ignore_case_args.clone()),
        ),
        ("<App B a />;", "<App a B />;", Some(ignore_case_args.clone())),
        ("<App B A c />;", "<App A B c />;", Some(ignore_case_args)),
        (r#"<App c="a" a="c" b="b" />;"#, r#"<App a="c" b="b" c="a" />;"#, None),
        (
            r#"<App {...this.props} c="a" a="c" b="b" />;"#,
            r#"<App {...this.props} a="c" b="b" c="a" />;"#,
            None,
        ),
        (
            r#"<App d="d" b="b" {...this.props} c="a" a="c" />;"#,
            r#"<App b="b" d="d" {...this.props} a="c" c="a" />;"#,
            None,
        ),
        (
            "
                    <App
                      a={true}
                      z
                      r
                      _onClick={function(){}}
                      onHandle={function(){}}
                      {...this.props}
                      b={false}
                      {...otherProps}
                    >
                      {test}
                    </App>
                  ",
            "
                    <App
                      _onClick={function(){}}
                      a={true}
                      onHandle={function(){}}
                      r
                      z
                      {...this.props}
                      b={false}
                      {...otherProps}
                    >
                      {test}
                    </App>
                  ",
            None,
        ),
        (
            "<App b={2} c={3} d={4} e={5} f={6} g={7} h={8} i={9} j={10} k={11} a={1} />",
            "<App a={1} b={2} c={3} d={4} e={5} f={6} g={7} h={8} i={9} j={10} k={11} />",
            None,
        ),
        (
            "
                    <List
                      className={className}
                      onStageAnswer={onStageAnswer}
                      onCommitAnswer={onCommitAnswer}
                      isFocused={isFocused}
                      direction={direction}
                      allowMultipleSelection={allowMultipleSelection}
                      measureLongestChildNode={measureLongestChildNode}
                      layoutItemsSize={layoutItemsSize}
                      handleAppScroll={handleAppScroll}
                      isActive={isActive}
                      resetSelection={resetSelection}
                      onKeyboardChoiceHovered={onKeyboardChoiceHovered}
                      keyboardShortcutType
                    />
                  ",
            "
                    <List
                      allowMultipleSelection={allowMultipleSelection}
                      className={className}
                      direction={direction}
                      handleAppScroll={handleAppScroll}
                      isActive={isActive}
                      isFocused={isFocused}
                      keyboardShortcutType
                      layoutItemsSize={layoutItemsSize}
                      measureLongestChildNode={measureLongestChildNode}
                      onCommitAnswer={onCommitAnswer}
                      onKeyboardChoiceHovered={onKeyboardChoiceHovered}
                      onStageAnswer={onStageAnswer}
                      resetSelection={resetSelection}
                    />
                  ",
            None,
        ),
        (
            "
                    <CreateNewJob
                      closed={false}
                      flagOptions={flagOptions}
                      jobHeight={300}
                      jobWidth={200}
                      campaign='Some Campaign name'
                      campaignStart={moment('2018-07-28 00:00:00')}
                      campaignFinish={moment('2018-09-01 00:00:00')}
                      jobNumber={'Job Number can be a String'}
                      jobTemplateOptions={jobTemplateOptions}
                      numberOfPages={30}
                      onChange={onChange}
                      onClose={onClose}
                      spreadSheetTemplateOptions={spreadSheetTemplateOptions}
                      stateMachineOptions={stateMachineOptions}
                      workflowTemplateOptions={workflowTemplateOptions}
                      workflowTemplateSteps={workflowTemplateSteps}
                      description='Some description for this job'
            
                      jobTemplate='1'
                      stateMachine='1'
                      flag='1'
                      spreadSheetTemplate='1'
                      workflowTemplate='1'
                      validation={validation}
                      onSubmit={onSubmit}
                    />
                  ",
            "
                    <CreateNewJob
                      campaign='Some Campaign name'
                      campaignFinish={moment('2018-09-01 00:00:00')}
                      campaignStart={moment('2018-07-28 00:00:00')}
                      closed={false}
                      description='Some description for this job'
                      flag='1'
                      flagOptions={flagOptions}
                      jobHeight={300}
                      jobNumber={'Job Number can be a String'}
                      jobTemplate='1'
                      jobTemplateOptions={jobTemplateOptions}
                      jobWidth={200}
                      numberOfPages={30}
                      onChange={onChange}
                      onClose={onClose}
                      onSubmit={onSubmit}
                      spreadSheetTemplate='1'
            
                      spreadSheetTemplateOptions={spreadSheetTemplateOptions}
                      stateMachine='1'
                      stateMachineOptions={stateMachineOptions}
                      validation={validation}
                      workflowTemplate='1'
                      workflowTemplateOptions={workflowTemplateOptions}
                      workflowTemplateSteps={workflowTemplateSteps}
                    />
                  ",
            None,
        ),
        (
            r#"<App key="key" b c="c" />"#,
            r#"<App key="key" c="c" b />"#,
            Some(reserved_first_with_shorthand_last.clone()),
        ),
        (
            r#"<App ref="ref" key="key" isShorthand veryLastAttribute="yes" />"#,
            r#"<App key="key" ref="ref" veryLastAttribute="yes" isShorthand />"#,
            Some(reserved_first_with_shorthand_last),
        ),
        ("<App a z onFoo onBar />;", "<App a z onBar onFoo />;", Some(callbacks_last_args.clone())),
        ("<App a onBar onFoo z />;", "<App a z onBar onFoo />;", Some(callbacks_last_args.clone())),
        (r#"<App a="a" b />;"#, r#"<App b a="a" />;"#, Some(shorthand_first_args.clone())),
        (r#"<App z x a="a" />;"#, r#"<App x z a="a" />;"#, Some(shorthand_first_args)),
        (r#"<App b a="a" />;"#, r#"<App a="a" b />;"#, Some(shorthand_last_args.clone())),
        (
            r#"<App a="a" onBar onFoo z x />;"#,
            r#"<App a="a" onBar onFoo x z />;"#,
            Some(shorthand_last_args),
        ),
        ("<App b a />;", "<App a b />;", Some(sort_alphabetically_args)),
        ("<App a key={1} />", "<App key={1} a />", Some(reserved_first_as_boolean_args.clone())),
        (
            r#"<div a dangerouslySetInnerHTML={{__html: "EPR"}} />"#,
            r#"<div dangerouslySetInnerHTML={{__html: "EPR"}} a />"#,
            Some(reserved_first_as_boolean_args.clone()),
        ),
        (
            r#"<App ref="r" key={2} b />"#,
            r#"<App key={2} ref="r" b />"#,
            Some(reserved_first_as_boolean_args.clone()),
        ),
        (
            "<App key={2} b a />",
            "<App key={2} a b />",
            Some(reserved_first_as_boolean_args.clone()),
        ),
        ("<App b a />", "<App a b />", Some(reserved_first_as_boolean_args.clone())),
        (
            r#"<App dangerouslySetInnerHTML={{__html: "EPR"}} e key={2} b />"#,
            r#"<App key={2} b dangerouslySetInnerHTML={{__html: "EPR"}} e />"#,
            Some(reserved_first_as_boolean_args.clone()),
        ),
        (
            "<App key={3} children={<App />} />",
            "<App children={<App />} key={3} />",
            Some(reserved_first_as_array_args),
        ),
        (
            r#"<App z ref="r" />"#,
            r#"<App ref="r" z />"#,
            Some(reserved_first_with_no_sort_alphabetically_args),
        ),
        ("<App onBar z />;", "<App z onBar />;", Some(reserved_first_and_callbacks_last_args)),
        // Reserved prop `key` after regular prop `as` on non-DOM component.
        (
            r#"<Box as="span" key={index} />"#,
            r#"<Box key={index} as="span" />"#,
            Some(reserved_first_as_boolean_args),
        ),
        (
            "
                    <App
                      a
                      b={{
                        bB: 1,
                      }}
                    />
                  ",
            "
                    <App
                      b={{
                        bB: 1,
                      }}
                      a
                    />
                  ",
            Some(multiline_first_args),
        ),
        (
            "
                    <App
                      a={1}
                      b={{
                        bB: 1,
                      }}
                      c
                    />
                  ",
            "
                    <App
                      c
                      b={{
                        bB: 1,
                      }}
                      a={1}
                    />
                  ",
            Some(multiline_and_shorthand_first_args),
        ),
        (
            "
                    <App
                      a={{
                        aA: 1,
                      }}
                      b
                    />
                  ",
            "
                    <App
                      b
                      a={{
                        aA: 1,
                      }}
                    />
                  ",
            Some(multiline_last_args),
        ),
        (
            r#"
                    <App
                      a={{
                        aA: 1,
                      }}
                      b
                      inline={1}
                      onClick={() => ({
                        c: 1
                      })}
                      d="dD"
                      e={() => ({
                        eE: 1
                      })}
                      f
                    />
                  "#,
            r#"
                    <App
                      d="dD"
                      inline={1}
                      a={{
                        aA: 1,
                      }}
                      e={() => ({
                        eE: 1
                      })}
                      b
                      f
                      onClick={() => ({
                        c: 1
                      })}
                    />
                  "#,
            Some(multiline_and_shorthand_and_callback_last_args),
        ),
        (
            r#"
                    <Typography
                      float
                      className={classNames(classes.inputWidth, {
                        [classes.noBorder]: isActive === "values",
                      })}
                      disabled={isDisabled}
                      initialValue={computePercentage(number, count)}
                      InputProps={{
                        ...customInputProps,
                      }}
                      key={index}
                      isRequired
                      {...sharedTypographyProps}
                      ref={textRef}
                      min="0"
                      name="fieldName"
                      placeholder={getTranslation("field")}
                      onValidate={validate}
                      inputProps={{
                        className: inputClassName,
                      }}
                      outlined
                      {...rest}
                    />
                  "#,
            r#"
                    <Typography
                      key={index}
                      float
                      isRequired
                      disabled={isDisabled}
                      initialValue={computePercentage(number, count)}
                      className={classNames(classes.inputWidth, {
                        [classes.noBorder]: isActive === "values",
                      })}
                      InputProps={{
                        ...customInputProps,
                      }}
                      {...sharedTypographyProps}
                      ref={textRef}
                      outlined
                      min="0"
                      name="fieldName"
                      placeholder={getTranslation("field")}
                      inputProps={{
                        className: inputClassName,
                      }}
                      onValidate={validate}
                      {...rest}
                    />
                  "#,
            Some(
                serde_json::json!([ { "multiline": "last", "shorthandFirst": true, "callbacksLast": true, "reservedFirst": true, "ignoreCase": true, }, ]),
            ),
        ),
        // Spread boundary fix tests: sorting respects spread boundaries.
        // Only the group after spread is unsorted.
        ("<App b {...rest} c a />;", "<App b {...rest} a c />;", None),
        // Both groups are unsorted, both get fixed independently.
        ("<App d c {...rest} b a />;", "<App c d {...rest} a b />;", None),
        // Multiple spreads, unsorted middle group.
        ("<App a {...x} d c {...y} e />;", "<App a {...x} c d {...y} e />;", None),
        // Multiple spreads, all groups unsorted.
        ("<App b a {...x} d c {...y} f e />;", "<App a b {...x} c d {...y} e f />;", None),
        // Unsorted only in last group after multiple spreads.
        ("<App a {...x} b {...y} d c />;", "<App a {...x} b {...y} c d />;", None),
        // Locale-aware fix tests:
        // Slovak: ä sorts between a and b, so "b ä" → "ä b".
        (r"<App b ä />", r"<App ä b />", Some(serde_json::json!([{ "locale": "sk" }]))),
        // Slovak: č sorts between c and d, so "d č" → "č d".
        (r"<App d č />", r"<App č d />", Some(serde_json::json!([{ "locale": "sk" }]))),
        // sortFirst: move className before name.
        (
            r#"<App name="John" className="test" />"#,
            r#"<App className="test" name="John" />"#,
            Some(sort_first_args.clone()),
        ),
        // sortFirst: multiple — reorder className before id.
        (
            r#"<App id="test" className="test" name="John" />"#,
            r#"<App className="test" id="test" name="John" />"#,
            Some(sort_first_multiple_args.clone()),
        ),
        // sortFirst: move className to front.
        (
            r#"<App a className="test" b />"#,
            r#"<App className="test" a b />"#,
            Some(sort_first_args.clone()),
        ),
        // sortFirst + reservedFirst: sortFirst takes priority over reserved.
        (
            r#"<App key={0} className="test" name="John" />"#,
            r#"<App className="test" key={0} name="John" />"#,
            Some(sort_first_with_reserved_first_args),
        ),
        // sortFirst + shorthandFirst: sortFirst takes priority.
        (
            r#"<App a className="test" name="John" />"#,
            r#"<App className="test" a name="John" />"#,
            Some(sort_first_with_shorthand_first_args),
        ),
        // sortFirst + callbacksLast: className first, callbacks still last.
        (
            r#"<App name="John" onClick={handleClick} className="test" />"#,
            r#"<App className="test" name="John" onClick={handleClick} />"#,
            Some(sort_first_with_callbacks_last_args),
        ),
        // sortFirst + multiline first.
        (
            r#"
                    <App
                      name="John"
                      className="test"
                      data={{
                        test: 1,
                      }}
                    />
                  "#,
            r#"
                    <App
                      className="test"
                      data={{
                        test: 1,
                      }}
                      name="John"
                    />
                  "#,
            Some(sort_first_with_multiline_first_args),
        ),
        // sortFirst + ignoreCase: classname matches className.
        (
            r#"<App name="John" classname="test" />"#,
            r#"<App classname="test" name="John" />"#,
            Some(sort_first_with_ignore_case_args),
        ),
        // sortFirst: regular props after sortFirst still sorted alphabetically.
        (
            r#"<App className="test" id="test" tel={5555555} name="John" />"#,
            r#"<App className="test" id="test" name="John" tel={5555555} />"#,
            Some(sort_first_multiple_args.clone()),
        ),
        // sortFirst: duplicate prop with wrong sortFirst order.
        (
            r#"<App id="test" className="test" id="test2" />"#,
            r#"<App className="test" id="test" id="test2" />"#,
            Some(sort_first_multiple_args),
        ),
        // ── Comment-handling fix tests ───────────────────────────
        // Test 1: standalone comment between c and f → c absorbs fofof+f,
        // has_comment=true so c(+group) sorts to end.
        (
            r#"<foo
  m={0}
  n={0} // this is n
  o={0}
  c={0} // this is c
  // fofof
  f={0} // this is f
  a={0}
  b={0}
  d={0}
/>"#,
            r#"<foo
  a={0}
  b={0}
  d={0}
  m={0}
  n={0} // this is n
  o={0}
  c={0} // this is c
  // fofof
  f={0} // this is f
/>"#,
            None,
        ),
        // Test 2: same-line line comments travel with their attribute,
        // no grouping needed, pure alphabetical sort.
        (
            r#"<foo
  m={0}
  n={0} // this is n
  o={0}
  c={0} // this is c
  f={0} // this is f
  e={0}
  a={0}
  b={0}
  d={0}
/>"#,
            r#"<foo
  a={0}
  b={0}
  c={0} // this is c
  d={0}
  e={0}
  f={0} // this is f
  m={0}
  n={0} // this is n
  o={0}
/>"#,
            None,
        ),
        // Test 3: d absorbs (// comment for d and aa + aa),
        // c absorbs (// comment for c and e + e).
        (
            r#"<foo
  a1={0}
  g={0}
  d={0} // comment for d
  // comment for d and aa
  aa={0}
  c={0} // comment for c
  // comment for c and e
  e={1}
  ab={1} // comment for ab
  f={0}
/>"#,
            r#"<foo
  a1={0}
  ab={1} // comment for ab
  f={0}
  g={0}
  c={0} // comment for c
  // comment for c and e
  e={1}
  d={0} // comment for d
  // comment for d and aa
  aa={0}
/>"#,
            None,
        ),
        // Test 4: ab absorbs (// comment for ab and f + f),
        // c absorbs (// comment for c and e + e).
        (
            r#"<foo
  a1={0}
  ab={1}
  // comment for ab and f
  f={0}
  g={0}
  c={0} // comment for c
  // comment for c and e
  e={1}
  d={0}
  aa={1} // comment for aa
/>"#,
            r#"<foo
  a1={0}
  aa={1} // comment for aa
  d={0}
  g={0}
  ab={1}
  // comment for ab and f
  f={0}
  c={0} // comment for c
  // comment for c and e
  e={1}
/>"#,
            None,
        ),
        // Test 5: inline block comment → b absorbs /* comment */ + ab,
        // has_comment=true so b(+group) sorts to end.
        (
            "<foo a={0} b={1} /* comment for b and ab */ ab={1} aa={0} />",
            "<foo a={0} aa={0} b={1} /* comment for b and ab */ ab={1} />",
            None,
        ),
        // Test 6: block comment after last attr (callbacksLast),
        // onEdit absorbs /* onDelete={onEdit} */, sorts to end.
        (
            r#"<ReactJson src={rowResult} name="data" collapsed={4} collapseStringsAfterLength={60} onEdit={onEdit} /* onDelete={onEdit} */ />"#,
            r#"<ReactJson collapseStringsAfterLength={60} collapsed={4} name="data" src={rowResult} onEdit={onEdit} /* onDelete={onEdit} */ />"#,
            Some(callbacks_last_args),
        ),
    ];

    Tester::new(JsxSortProps::NAME, JsxSortProps::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
