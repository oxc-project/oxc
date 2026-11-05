use oxc_span::{LabeledSpan, Span};

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

/// Moves `labels` pointing into `normalize_newlines(source, ['\r'])` onto `source`.
///
/// Entries that normalize line endings before parsing collapse each `\r\n` into one byte,
/// so a label shifts right by the CRLFs before it.
pub fn restore_crlf_labels(labels: &mut [LabeledSpan], source: &str) {
    let restore = |offset: u32| {
        let mut offset = offset as usize;
        for (pos, _) in source.match_indices("\r\n") {
            if pos >= offset {
                break;
            }
            offset += 1;
        }
        u32::try_from(offset).unwrap_or(u32::MAX)
    };
    for label in labels {
        let span = label.span();
        let span = Span::new(restore(span.start), restore(span.end));
        let text = label.label().map(ToString::to_string);
        *label = if label.primary() {
            LabeledSpan::new_primary_with_span(text, span)
        } else {
            LabeledSpan::new_with_span(text, span)
        };
    }
}
