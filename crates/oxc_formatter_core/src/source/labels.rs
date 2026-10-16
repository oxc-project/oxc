use oxc_span::LabeledSpan;

/// Moves `labels` pointing into `tail`, a suffix of `source`, onto `source`.
///
/// Entries parse a trimmed tail (e.g. after [`crate::spec::split_bom`]),
/// but their diagnostics are reported against the original source.
pub fn rebase_labels(labels: &mut [LabeledSpan], source: &str, tail: &str) {
    debug_assert!(source.ends_with(tail));
    let offset = u32::try_from(source.len() - tail.len()).unwrap_or(u32::MAX);
    for label in labels {
        label.set_span_offset(label.offset().saturating_add(offset));
    }
}
