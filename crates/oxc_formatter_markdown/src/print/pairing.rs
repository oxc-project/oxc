//! Delimiter pairing check: the parser's resolver run on the text about to be printed.
//!
//! Marker normalization (`_` to `*`, `__` to `**`) and line joining change the neighbors of every
//! `*` / `_` / `~` run, and with them which runs pair (flanking, the rule of three, family order).
//! Instead of predicting that, the printed text is tokenized into runs and handed to
//! `oxc_markdown_parser::attention::pairs`; the result must be the tree being printed.
//! When it is not, the paragraph is printed again with its markers as written.

use oxc_markdown_parser::{
    Constructs,
    attention::{Pair, Run, pairs},
    escapes_next,
};

/// A printed-offset annotation recorded while collecting phrasing content.
#[derive(Clone, Copy, Debug)]
pub enum Mark {
    /// An emphasis / strong / strikethrough node: the offsets of its opening and closing marker.
    Delimiter { open: u32, close: u32, strong: bool },
    /// Text whose delimiter characters never pair (code span, HTML, autolink, math, ...).
    Opaque { start: u32, end: u32 },
    /// A link's text: its runs resolve on their own (bracket level).
    Group { start: u32, end: u32 },
}

/// A delimiter run of the printed text: where it is, and which group it resolves in.
struct PrintedRun {
    at: u32,
    run: Run,
    group: usize,
}

/// Whether the runs of `flat` (the printed text) form exactly the nodes in `marks`.
pub fn consistent(flat: &str, marks: &[Mark]) -> bool {
    let constructs = Constructs::markdown();
    let to_u32 = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    // The whole text is the outermost group; a run or a node belongs to the innermost one
    let mut groups: Vec<(u32, u32)> = vec![(0, to_u32(flat.len()))];
    groups.extend(marks.iter().filter_map(|m| match m {
        Mark::Group { start, end } => Some((*start, *end)),
        _ => None,
    }));
    let group_of = |offset: u32| -> usize {
        groups
            .iter()
            .enumerate()
            .filter(|(_, (start, end))| *start <= offset && offset < *end)
            .min_by_key(|(_, (start, end))| end - start)
            .map_or(0, |(i, _)| i)
    };

    // Every run outside opaque text (recorded in print order, so a cursor suffices), escapes skipped
    let mut opaque = marks.iter().filter_map(|m| match m {
        Mark::Opaque { start, end } => Some((*start, *end)),
        _ => None,
    });
    let mut next_opaque = opaque.next();
    let bytes = flat.as_bytes();
    let mut runs: Vec<PrintedRun> = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let at = to_u32(i);
        if let Some((start, end)) = next_opaque
            && at >= start
        {
            i = i.max(end as usize);
            next_opaque = opaque.next();
            continue;
        }
        match bytes[i] {
            b'\\' if escapes_next(bytes, i) => i += 2,
            marker @ (b'*' | b'_' | b'~') => {
                let len = bytes[i..].iter().take_while(|&&b| b == marker).count();
                let prev = flat[..i].chars().next_back();
                let next = flat[i + len..].chars().next();
                let run = Run::new(marker, len, prev, next, &constructs);
                runs.push(PrintedRun { at, run, group: group_of(at) });
                i += len;
            }
            _ => i += 1,
        }
    }

    for group in 0..groups.len() {
        let members: Vec<&PrintedRun> = runs.iter().filter(|r| r.group == group).collect();
        let index_of = |offset: u32| -> Option<usize> {
            members.iter().position(|r| r.at <= offset && offset < r.at + to_u32(r.run.len))
        };
        let mut expected: Vec<Pair> = marks
            .iter()
            .filter_map(|m| match *m {
                Mark::Delimiter { open, close, strong } if group_of(open) == group => {
                    Some(Pair { opener: index_of(open)?, closer: index_of(close)?, strong })
                }
                _ => None,
            })
            .collect();
        expected.sort_unstable();
        let group_runs: Vec<Run> = members.iter().map(|r| r.run).collect();
        if pairs(&group_runs, group != 0) != expected {
            return false;
        }
    }
    true
}
