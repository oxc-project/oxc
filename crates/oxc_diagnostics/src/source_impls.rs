//! Default trait implementations for [`SourceCode`].
#[cfg(test)]
use std::str::from_utf8;
use std::{collections::VecDeque, sync::Arc};

use crate::SourceCode;
use oxc_span::Span;

#[derive(Debug)]
pub struct SpanContents<'a> {
    data: &'a [u8],
    span: Span,
    line: usize,
    column: usize,
    line_count: usize,
}

impl<'a> SpanContents<'a> {
    pub const fn new(
        data: &'a [u8],
        span: Span,
        line: usize,
        column: usize,
        line_count: usize,
    ) -> Self {
        Self { data, span, line, column, line_count }
    }

    pub const fn data(&self) -> &'a [u8] {
        self.data
    }

    pub const fn span(&self) -> &Span {
        &self.span
    }

    pub const fn line(&self) -> usize {
        self.line
    }

    pub const fn column(&self) -> usize {
        self.column
    }

    pub const fn line_count(&self) -> usize {
        self.line_count
    }
}

#[derive(Clone, Copy)]
struct ContextLines {
    before: usize,
    after: usize,
}

impl ContextLines {
    const fn new(before: usize, after: usize) -> Self {
        Self { before, after }
    }
}

/// The normalized, integer-only form of a [`Span`] query.
#[derive(Clone, Copy)]
struct SpanRequest {
    offset: usize,
    len: usize,
}

impl SpanRequest {
    fn new(span: Span) -> Self {
        Self { offset: span.start as usize, len: span.size() as usize }
    }

    /// Boundary between the bulk prefix scan and the detailed span scan.
    /// Never splits a CRLF pair.
    fn prefix_end(self, input: &[u8]) -> usize {
        let mut end = self.offset.saturating_sub(1).min(input.len());
        if end > 0 && input[end - 1] == b'\r' {
            end -= 1;
        }
        end
    }

    /// First byte at which the detailed scan has consumed the requested span.
    fn end_threshold(self) -> usize {
        self.offset.saturating_add(self.len).saturating_sub(1)
    }

    /// First line break that can belong to the trailing context.
    fn trailing_break_threshold(self) -> usize {
        self.offset.saturating_add(self.len.saturating_sub(1))
    }
}

/// A logical line break. `start == end` for LF or CR and differs by one for
/// CRLF, allowing all scanners to share the same newline handling.
#[derive(Clone, Copy)]
struct LineBreak {
    start: usize,
    end: usize,
}

impl LineBreak {
    fn ending_at(input: &[u8], end: usize) -> Self {
        let start =
            if end > 0 && input[end] == b'\n' && input[end - 1] == b'\r' { end - 1 } else { end };
        Self { start, end }
    }

    const fn next_line_start(self) -> usize {
        self.end + 1
    }

    const fn shifted(self, offset: usize) -> Self {
        Self { start: self.start + offset, end: self.end + offset }
    }
}

/// Iterator over logical line breaks in a slice. It consumes CRLF as one item,
/// so callers do not need their own "skip the LF" branches.
struct LineBreaks<'a> {
    input: &'a [u8],
    positions: memchr::Memchr2<'a>,
}

impl<'a> LineBreaks<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self { input, positions: memchr::memchr2_iter(b'\r', b'\n', input) }
    }
}

impl Iterator for LineBreaks<'_> {
    type Item = LineBreak;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let start = self.positions.next()?;
            // The CR already represents this CRLF pair.
            if start > 0 && self.input[start] == b'\n' && self.input[start - 1] == b'\r' {
                continue;
            }
            let end = if self.input[start] == b'\r' && self.input.get(start + 1) == Some(&b'\n') {
                start + 1
            } else {
                start
            };
            return Some(LineBreak { start, end });
        }
    }
}

/// The retained leading-context lines and the absolute line number of the
/// first one. The size invariant lives here instead of being repeated by each
/// scanner.
struct LeadingContext {
    limit: usize,
    start_line: usize,
    line_starts: RetainedLineStarts,
}

/// Storage for the retained line starts. The renderer's default one-line
/// context stays inline instead of allocating a `VecDeque` for every span.
enum RetainedLineStarts {
    One(Option<usize>),
    Many(VecDeque<usize>),
}

impl LeadingContext {
    fn new(limit: usize) -> Self {
        let line_starts = if limit == 1 {
            RetainedLineStarts::One(None)
        } else {
            RetainedLineStarts::Many(VecDeque::new())
        };
        Self { limit, start_line: 0, line_starts }
    }

    fn one(start_line: usize, line_start: Option<usize>) -> Self {
        Self { limit: 1, start_line, line_starts: RetainedLineStarts::One(line_start) }
    }

    fn len(&self) -> usize {
        match &self.line_starts {
            RetainedLineStarts::One(line_start) => usize::from(line_start.is_some()),
            RetainedLineStarts::Many(line_starts) => line_starts.len(),
        }
    }

    fn first(&self) -> Option<usize> {
        match &self.line_starts {
            RetainedLineStarts::One(line_start) => *line_start,
            RetainedLineStarts::Many(line_starts) => line_starts.front().copied(),
        }
    }

    #[cfg(test)]
    fn last(&self) -> Option<usize> {
        match &self.line_starts {
            RetainedLineStarts::One(line_start) => *line_start,
            RetainedLineStarts::Many(line_starts) => line_starts.back().copied(),
        }
    }

    fn push(&mut self, line_start: usize) {
        match &mut self.line_starts {
            RetainedLineStarts::One(retained) => {
                if retained.replace(line_start).is_some() {
                    self.start_line += 1;
                }
            }
            RetainedLineStarts::Many(line_starts) => {
                line_starts.push_back(line_start);
                if line_starts.len() > self.limit {
                    self.start_line += 1;
                    line_starts.pop_front();
                }
            }
        }
    }

    fn starting_offset(&self, span_offset: usize) -> usize {
        self.first().unwrap_or(if self.limit == 0 { span_offset } else { 0 })
    }

    fn append_to(self, target: &mut Vec<usize>) {
        match self.line_starts {
            RetainedLineStarts::One(line_start) => target.extend(line_start),
            RetainedLineStarts::Many(line_starts) => target.extend(line_starts),
        }
    }
}

/// State produced by scanning the source prefix before a span.
struct PrefixScan {
    line_count: usize,
    leading: LeadingContext,
    current_line_start: usize,
}

impl PrefixScan {
    /// Scan the prefix before a span, retaining only its leading context lines.
    #[inline]
    fn new(input: &[u8], end: usize, context_lines_before: usize) -> Self {
        let prefix = &input[..end];
        if context_lines_before == 1 {
            return Self::one_context_line(prefix);
        }

        let mut scan = Self {
            line_count: 0,
            leading: LeadingContext::new(context_lines_before),
            current_line_start: 0,
        };
        for line_break in LineBreaks::new(prefix) {
            scan.line_count += 1;
            scan.leading.push(scan.current_line_start);
            scan.current_line_start = line_break.next_line_start();
        }
        scan
    }

    /// The graphical handler's default. Keep the one retained line start in a
    /// scalar while scanning instead of updating a `VecDeque` for every line.
    fn one_context_line(prefix: &[u8]) -> Self {
        let mut line_count = 0;
        let mut current_line_start = 0;
        let mut previous_line_start = None;

        if memchr::memchr(b'\r', prefix).is_none() {
            // Most source files only use LF. Count all breaks in one SIMD pass,
            // then recover the two line starts the caller needs from the end.
            line_count = bytecount::count(prefix, b'\n');
            if let Some(last_break) = memchr::memrchr(b'\n', prefix) {
                current_line_start = last_break + 1;
                previous_line_start =
                    Some(memchr::memrchr(b'\n', &prefix[..last_break]).map_or(0, |pos| pos + 1));
            }
        } else {
            for line_break in LineBreaks::new(prefix) {
                line_count += 1;
                previous_line_start = Some(current_line_start);
                current_line_start = line_break.next_line_start();
            }
        }

        let leading = LeadingContext::one(line_count.saturating_sub(1), previous_line_start);
        Self { line_count, leading, current_line_start }
    }
}

/// Tracks when the requested span has been consumed and how much trailing
/// context has subsequently been collected.
#[derive(Default)]
struct TrailingContext {
    active: bool,
    saw_newline: bool,
    line_count: usize,
}

impl TrailingContext {
    fn activate(&mut self) {
        self.active = true;
    }

    fn record_break(&mut self) {
        if self.saw_newline {
            self.line_count += 1;
        } else {
            self.saw_newline = true;
        }
    }

    fn is_complete(&self, requested_lines: usize) -> bool {
        self.line_count >= requested_lines
    }
}

/// A single span read. Keeping its counters together makes the phase
/// transitions and the final payload invariants explicit.
struct SpanReader<'a> {
    input: &'a [u8],
    request: SpanRequest,
    context: ContextLines,
    leading: LeadingContext,
    trailing: TrailingContext,
    line_count: usize,
    current_line_start: usize,
    start_column: usize,
    offset: usize,
}

impl<'a> SpanReader<'a> {
    #[cfg(test)]
    fn from_span(
        input: &'a [u8],
        span: Span,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Self {
        Self::new(
            input,
            SpanRequest::new(span),
            ContextLines::new(context_lines_before, context_lines_after),
        )
    }

    fn new(input: &'a [u8], request: SpanRequest, context: ContextLines) -> Self {
        let offset = request.prefix_end(input);
        let PrefixScan { line_count, leading, current_line_start } =
            PrefixScan::new(input, offset, context.before);
        Self {
            input,
            request,
            context,
            leading,
            trailing: TrailingContext::default(),
            line_count,
            current_line_start,
            start_column: offset - current_line_start,
            offset,
        }
    }

    fn read(mut self) -> Option<SpanContents<'a>> {
        while self.offset < self.input.len() {
            let byte = self.input[self.offset];
            if matches!(byte, b'\r' | b'\n') {
                let end = if byte == b'\r' && self.input.get(self.offset + 1) == Some(&b'\n') {
                    self.offset + 1
                } else {
                    self.offset
                };
                if self.consume_line_break(end) {
                    self.offset = end + 1;
                    break;
                }
                self.offset = end;
            } else if self.offset < self.request.offset {
                self.start_column += 1;
            }

            if self.offset >= self.request.end_threshold() {
                self.trailing.activate();
                if self.trailing.is_complete(self.context.after) {
                    self.offset += 1;
                    break;
                }
            }
            self.offset += 1;
        }
        self.finish()
    }

    /// Returns whether the requested trailing context is complete.
    fn consume_line_break(&mut self, end: usize) -> bool {
        self.line_count += 1;
        if end < self.request.offset {
            self.start_column = 0;
            self.leading.push(self.current_line_start);
        } else if end >= self.request.trailing_break_threshold() && self.trailing.active {
            self.start_column = 0;
            self.trailing.record_break();
            if self.trailing.is_complete(self.context.after) {
                return true;
            }
        }
        self.current_line_start = end + 1;
        false
    }

    fn finish(self) -> Option<SpanContents<'a>> {
        if self.offset < self.request.end_threshold() {
            return None;
        }

        let start = self.leading.starting_offset(self.request.offset);
        // A zero-length span starting just past the end of the input reaches
        // the threshold but has no content to slice.
        let Some(data) = self.input.get(start..self.offset) else {
            return None;
        };
        let span_start = u32::try_from(start).ok()?;
        let span_len = u32::try_from(self.offset - start).ok()?;
        Some(SpanContents::new(
            data,
            Span::sized(span_start, span_len),
            self.leading.start_line,
            if self.context.before == 0 { self.start_column } else { 0 },
            self.line_count,
        ))
    }
}

impl SpanContents<'_> {
    /// The 0-indexed line and column of an absolute source `offset` that lies
    /// within this payload, derived without re-reading the source.
    ///
    /// Equivalent to the `line()`/`column()` a fresh zero-context span read
    /// would report, but obtained by scanning only this payload's prefix up to
    /// `offset`. Returns `None` when `offset` is past this payload. Newline
    /// handling mirrors [`SpanReader`] (a `\r\n` pair
    /// and a lone `\r`/`\n` each count once), scanned with the same `memchr2`
    /// primitive so a long (e.g. minified) line stays cheap.
    // Only the `fancy` graphical renderer consumes this today; without it the
    // method is dead in a lib-only build (CI lints `-D warnings`).
    pub(crate) fn line_column_at(&self, offset: usize) -> Option<(usize, usize)> {
        let data = self.data();
        let base = self.span().start as usize;
        let mut rel = offset.saturating_sub(base);
        // A label past the end of the payload is out of bounds; reject it rather
        // than clamping to a misleading end-of-snippet position.
        if rel > data.len() {
            return None;
        }
        // An offset landing on the `\n` of a `\r\n` pair sits inside an
        // unfinished break: `SpanReader` has not advanced the line at that
        // byte and reports the preceding `\r`. Normalize to that byte.
        if rel > 0 && rel < data.len() && data[rel - 1] == b'\r' && data[rel] == b'\n' {
            rel -= 1;
        }
        let mut line = self.line();
        // Byte index just past the most recent line break — the start of the
        // line `offset` falls on. `None` until the first break is seen, meaning
        // `offset` is still on this payload's first line.
        let mut line_start: Option<usize> = None;
        for line_break in LineBreaks::new(&data[..rel]) {
            line += 1;
            line_start = Some(line_break.next_line_start());
        }
        Some(match line_start {
            // A later line, which by definition starts at column 0.
            Some(start) => (line, rel - start),
            // Still on the first line: offset from this payload's start column.
            None => (line, self.column() + rel),
        })
    }
}

/// Lazily built line-start index over a contiguous source buffer.
struct LineIndex<'a> {
    input: &'a [u8],
    /// Starts (byte offsets) of consecutive lines, the first of which is line
    /// number `base_line`; covers every line whose start lies in
    /// `[line_starts[0], frontier]`. Empty until the first query seeds it.
    line_starts: Vec<usize>,
    /// 0-indexed line number of `line_starts[0]`.
    base_line: usize,
    /// Bytes in `[0, frontier)` have been scanned: every line break there is
    /// either recorded in `line_starts` or (before `line_starts[0]`) summed
    /// into `base_line`. Never splits a `\r\n` pair.
    frontier: usize,
}

impl<'a> LineIndex<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self { input, line_starts: Vec::new(), base_line: 0, frontier: 0 }
    }

    fn is_empty(&self) -> bool {
        self.line_starts.is_empty()
    }

    fn origin(&self) -> Option<usize> {
        self.line_starts.first().copied()
    }

    /// First query: retain the line starts in its leading context window and
    /// use them as the origin of the reusable index.
    fn init(&mut self, cut: usize, context_lines_before: usize) {
        let PrefixScan { line_count, leading, current_line_start } =
            PrefixScan::new(self.input, cut, context_lines_before);
        debug_assert_eq!(leading.start_line + leading.len(), line_count);
        self.base_line = leading.start_line;
        self.line_starts.reserve(leading.len() + 8);
        leading.append_to(&mut self.line_starts);
        self.line_starts.push(current_line_start);
        self.frontier = cut;
    }

    /// Extend the index so every break in `[0, target)` is recorded, with one
    /// bulk `memchr` pass. (A `read_span`-shaped `target` never splits a
    /// `\r\n` pair, but hold the pair back a byte if one would be.)
    fn cover(&mut self, mut target: usize) {
        if target > self.frontier
            && self.input[target - 1] == b'\r'
            && self.input.get(target) == Some(&b'\n')
        {
            target -= 1;
        }
        if target <= self.frontier {
            return;
        }
        for line_break in LineBreaks::new(&self.input[self.frontier..target]) {
            self.line_starts.push(line_break.shifted(self.frontier).next_line_start());
        }
        self.frontier = target;
    }

    /// Scan forward from `frontier` to the next line break, recording the
    /// line start after it. `None` at end of input (with `frontier` advanced
    /// there so the probe isn't repeated).
    fn extend(&mut self) -> Option<LineBreak> {
        if let Some(line_break) = LineBreaks::new(&self.input[self.frontier..]).next() {
            let line_break = line_break.shifted(self.frontier);
            self.line_starts.push(line_break.next_line_start());
            self.frontier = line_break.next_line_start();
            Some(line_break)
        } else {
            self.frontier = self.input.len();
            None
        }
    }

    /// 0-indexed line number of the line containing `offset`. Requires
    /// `line_starts[0] <= offset` and `offset <= frontier`.
    fn line_index_of(&self, offset: usize) -> usize {
        self.base_line + self.line_starts.partition_point(|&start| start <= offset) - 1
    }

    /// Byte offset where line number `line` starts. Requires the line to be
    /// indexed.
    fn line_start_of(&self, line: usize) -> usize {
        self.line_starts[line - self.base_line]
    }

    /// The break terminating line number `line` (which contains `pos`),
    /// extending the scan on demand.
    /// `None` when the line runs to end of input.
    fn break_ending_line(&mut self, line: usize, pos: usize) -> Option<LineBreak> {
        if let Some(&next_start) = self.line_starts.get(line + 1 - self.base_line) {
            let line_break = LineBreak::ending_at(self.input, next_start - 1);
            debug_assert!(line_break.start >= pos);
            return Some(line_break);
        }
        // `pos` is on the last indexed line; its terminator (if any) is at or
        // past the frontier.
        debug_assert!(pos <= self.frontier);
        self.extend()
    }
}

/// Leading-context state reconstructed from a [`LineIndex`].
struct IndexedLeadingContext {
    limit: usize,
    start_line: usize,
    len: usize,
}

impl IndexedLeadingContext {
    fn new(current_line: usize, limit: usize) -> Self {
        let start_line = current_line - limit.min(current_line);
        Self { limit, start_line, len: current_line - start_line }
    }

    fn push(&mut self) {
        self.len += 1;
        if self.len > self.limit {
            self.start_line += 1;
            self.len -= 1;
        }
    }

    fn starting_offset(&self, index: &LineIndex<'_>, span_offset: usize) -> usize {
        if self.len > 0 {
            index.line_start_of(self.start_line)
        } else if self.limit == 0 {
            span_offset
        } else {
            0
        }
    }
}

/// One span query replayed against a reusable [`LineIndex`].
struct IndexedReader<'index, 'source> {
    index: &'index mut LineIndex<'source>,
    request: SpanRequest,
    context: ContextLines,
    leading: IndexedLeadingContext,
    trailing: TrailingContext,
    line_count: usize,
    start_column: usize,
    position: usize,
}

impl<'index, 'source> IndexedReader<'index, 'source> {
    fn new(
        index: &'index mut LineIndex<'source>,
        request: SpanRequest,
        context: ContextLines,
        position: usize,
    ) -> Self {
        let line_count = index.line_index_of(position);
        Self {
            start_column: position - index.line_start_of(line_count),
            index,
            request,
            context,
            leading: IndexedLeadingContext::new(line_count, context.before),
            trailing: TrailingContext::default(),
            line_count,
            position,
        }
    }

    /// Jump line break to line break while preserving [`SpanReader`]'s exact
    /// edge-case behavior.
    fn read(mut self) -> Option<SpanContents<'source>> {
        let input = self.index.input;
        let window_end = loop {
            let line_break = self.index.break_ending_line(self.line_count, self.position);
            let run_end = line_break.map_or(input.len(), |line_break| line_break.start);
            if self.position < self.request.offset {
                self.start_column += self.request.offset.min(run_end) - self.position;
            }
            if run_end > self.request.end_threshold() && run_end > self.position {
                self.trailing.activate();
                if self.trailing.is_complete(self.context.after) {
                    break self.request.end_threshold().max(self.position) + 1;
                }
            }
            let Some(line_break) = line_break else {
                // No more breaks: the scan runs off the end of the input.
                break input.len();
            };
            if self.consume_line_break(line_break.end) {
                break line_break.next_line_start();
            }
            self.position = line_break.next_line_start();
        };
        self.finish(window_end)
    }

    fn consume_line_break(&mut self, end: usize) -> bool {
        self.line_count += 1;
        if end < self.request.offset {
            self.start_column = 0;
            self.leading.push();
        } else if end >= self.request.trailing_break_threshold() && self.trailing.active {
            self.start_column = 0;
            self.trailing.record_break();
            if self.trailing.is_complete(self.context.after) {
                return true;
            }
        }
        if end >= self.request.end_threshold() {
            self.trailing.activate();
            return self.trailing.is_complete(self.context.after);
        }
        false
    }

    fn finish(self, window_end: usize) -> Option<SpanContents<'source>> {
        if window_end < self.request.end_threshold() {
            return None;
        }
        let start = self.leading.starting_offset(self.index, self.request.offset);
        let Some(data) = self.index.input.get(start..window_end) else {
            return None;
        };
        let span_start = u32::try_from(start).ok()?;
        let span_len = u32::try_from(window_end - start).ok()?;
        Some(SpanContents::new(
            data,
            Span::sized(span_start, span_len),
            self.leading.start_line,
            if self.context.before == 0 { self.start_column } else { 0 },
            self.line_count,
        ))
    }
}

/// A line-break index over a contiguous source buffer, built with a single
/// forward scan and able to answer repeated span queries without re-reading
/// the source.
///
/// [`GraphicalReportHandler`] needs one span lookup per label plus one per
/// attempted snippet merge. A `SpanScanner` scans each source byte at most
/// once and records line starts for reuse. Queries that precede the index's
/// origin fall back to a standalone [`SpanReader`].
///
/// [`GraphicalReportHandler`]: crate::handlers::GraphicalReportHandler
pub struct SpanScanner<'a> {
    context: ContextLines,
    index: LineIndex<'a>,
}

impl<'a> SpanScanner<'a> {
    pub fn new(input: &'a [u8], context_lines_before: usize, context_lines_after: usize) -> Self {
        Self {
            context: ContextLines::new(context_lines_before, context_lines_after),
            index: LineIndex::new(input),
        }
    }

    /// Whether this scanner indexes `input`.
    pub fn is_for(&self, input: &[u8]) -> bool {
        std::ptr::eq(self.index.input, input)
    }

    /// Read a span while scanning only source bytes no earlier query scanned.
    pub fn read_span(&mut self, span: Span) -> Option<SpanContents<'a>> {
        let request = SpanRequest::new(span);
        let cut = request.prefix_end(self.index.input);
        if self.index.is_empty() {
            self.index.init(cut, self.context.before);
        } else {
            if cut < self.index.origin().expect("a non-empty index has an origin") {
                return self.read_unindexed(request);
            }
            self.index.cover(cut);
            // The query's leading context must not reach lines above the
            // index origin (only possible for spans out of sorted order).
            let cut_line = self.index.line_index_of(cut);
            if cut_line - self.context.before.min(cut_line) < self.index.base_line {
                return self.read_unindexed(request);
            }
        }
        IndexedReader::new(&mut self.index, request, self.context, cut).read()
    }

    fn read_unindexed(&self, request: SpanRequest) -> Option<SpanContents<'a>> {
        SpanReader::new(self.index.input, request, self.context).read()
    }
}

impl SourceCode for str {
    fn data(&self) -> &[u8] {
        self.as_bytes()
    }
}

/// Supports borrowed source text in generic contexts.
impl SourceCode for &str {
    fn data(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl SourceCode for String {
    fn data(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<T: ?Sized + SourceCode> SourceCode for Arc<T> {
    fn data(&self) -> &[u8] {
        self.as_ref().data()
    }

    fn name(&self) -> Option<&str> {
        self.as_ref().name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lf_prefix_fast_path_matches_generic_path() {
        let input = b"zero\none\n\ntwo\nthree\n";
        for cut in 0..=input.len() {
            let fast = PrefixScan::new(input, cut, 1);
            let generic = PrefixScan::new(input, cut, 2);
            assert_eq!(fast.line_count, generic.line_count, "cut={cut}");
            assert_eq!(fast.current_line_start, generic.current_line_start, "cut={cut}");
            assert_eq!(fast.leading.first(), generic.leading.last(), "cut={cut}");
            assert_eq!(fast.leading.start_line, fast.line_count.saturating_sub(1), "cut={cut}");
        }
    }

    #[test]
    fn basic() {
        let src = String::from("foo\n");
        let contents =
            SpanReader::from_span(src.as_bytes(), Span::sized(0, 4), 0, 0).read().unwrap();
        assert_eq!("foo\n", from_utf8(contents.data()).unwrap());
        assert_eq!(0, contents.line());
        assert_eq!(0, contents.column());
    }

    #[test]
    fn shifted() {
        let src = String::from("foobar");
        let contents =
            SpanReader::from_span(src.as_bytes(), Span::sized(3, 3), 1, 1).read().unwrap();
        assert_eq!("foobar", from_utf8(contents.data()).unwrap());
        assert_eq!(0, contents.line());
        assert_eq!(0, contents.column());
    }

    #[test]
    fn middle() {
        let src = String::from("foo\nbar\nbaz\n");
        let contents =
            SpanReader::from_span(src.as_bytes(), Span::sized(4, 4), 0, 0).read().unwrap();
        assert_eq!("bar\n", from_utf8(contents.data()).unwrap());
        assert_eq!(1, contents.line());
        assert_eq!(0, contents.column());
    }

    #[test]
    fn middle_of_line() {
        let src = String::from("foo\nbarbar\nbaz\n");
        let contents =
            SpanReader::from_span(src.as_bytes(), Span::sized(7, 4), 0, 0).read().unwrap();
        assert_eq!("bar\n", from_utf8(contents.data()).unwrap());
        assert_eq!(1, contents.line());
        assert_eq!(3, contents.column());
    }

    #[test]
    fn with_crlf() {
        let src = String::from("foo\r\nbar\r\nbaz\r\n");
        let contents =
            SpanReader::from_span(src.as_bytes(), Span::sized(5, 5), 0, 0).read().unwrap();
        assert_eq!("bar\r\n", from_utf8(contents.data()).unwrap());
        assert_eq!(1, contents.line());
        assert_eq!(0, contents.column());
    }

    #[test]
    fn with_context() {
        let src = String::from("xxx\nfoo\nbar\nbaz\n\nyyy\n");
        let contents =
            SpanReader::from_span(src.as_bytes(), Span::sized(8, 3), 1, 1).read().unwrap();
        assert_eq!("foo\nbar\nbaz\n", from_utf8(contents.data()).unwrap());
        assert_eq!(1, contents.line());
        assert_eq!(0, contents.column());
    }

    #[test]
    fn multiline_with_context() {
        let src = String::from("aaa\nxxx\n\nfoo\nbar\nbaz\n\nyyy\nbbb\n");
        let contents =
            SpanReader::from_span(src.as_bytes(), Span::sized(9, 11), 1, 1).read().unwrap();
        assert_eq!("\nfoo\nbar\nbaz\n\n", from_utf8(contents.data()).unwrap());
        assert_eq!(2, contents.line());
        assert_eq!(0, contents.column());
        let span = Span::sized(8, 14);
        assert_eq!(&span, contents.span());
    }

    #[test]
    fn zero_length_span_just_past_eof() {
        // Used to panic with a slice out-of-range instead of returning an
        // error (found by differential fuzzing).
        let src = String::from("a");
        assert!(SpanReader::from_span(src.as_bytes(), Span::sized(2, 0), 0, 0).read().is_none());
        let src = String::new();
        assert!(SpanReader::from_span(src.as_bytes(), Span::sized(1, 0), 0, 0).read().is_none());
    }

    #[test]
    fn multiline_with_context_line_start() {
        let src = String::from("one\ntwo\n\nthree\nfour\nfive\n\nsix\nseven\n");
        let contents =
            SpanReader::from_span(src.as_bytes(), Span::sized(2, 0), 2, 2).read().unwrap();
        assert_eq!("one\ntwo\n\n", from_utf8(contents.data()).unwrap());
        assert_eq!(0, contents.line());
        assert_eq!(0, contents.column());
        let span = Span::sized(0, 9);
        assert_eq!(&span, contents.span());
    }
}

#[cfg(test)]
mod line_column_tests {
    #![expect(
        clippy::cast_possible_truncation,
        reason = "deterministic fuzz inputs are tightly bounded"
    )]

    use super::*;

    /// Deterministic xorshift so any failure reproduces from a fixed seed.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        fn below(&mut self, n: usize) -> usize {
            (self.next() % n as u64) as usize
        }
    }

    /// Assert `SpanContents::line_column_at` matches a real
    /// `read_span(&(off, 0), 0, 0)` for an offset inside a
    /// `read_span(&(off, len), ctx, ctx)` payload — exactly how the graphical
    /// renderer consumes it. Full equivalence, including failure:
    /// `line_column_at` must return `None` iff the direct read returns `None`
    /// (a label past the snippet, e.g. `off == src.len() + 1`). Returns whether
    /// a case was actually checked (the context read succeeded).
    fn check(src: &str, off: usize, len: usize, ctx: usize) -> bool {
        let Some(contents) =
            SpanReader::from_span(src.as_bytes(), Span::sized(off as u32, len as u32), ctx, ctx)
                .read()
        else {
            return false;
        };
        let got = contents.line_column_at(off);

        match SpanReader::from_span(src.as_bytes(), Span::empty(off as u32), 0, 0).read() {
            Some(expected) => assert_eq!(
                got,
                Some((expected.line(), expected.column())),
                "mismatch for src={src:?} off={off} len={len} ctx={ctx}"
            ),
            None => assert_eq!(
                got, None,
                "accepted an out-of-bounds label for src={src:?} off={off} len={len} ctx={ctx}"
            ),
        }
        true
    }

    /// Exhaustively check every char-boundary offset of many small randomized
    /// sources — plus offsets one and two bytes past EOF — across LF / CRLF /
    /// lone-CR and multibyte alphabets.
    #[test]
    #[cfg_attr(
        miri,
        ignore = "equivalence fuzzer over safe, bounds-checked code — Miri finds no UB here \
                  and interprets it orders of magnitude slower; the derivation is still exercised \
                  under Miri by the normal snapshot tests"
    )]
    fn matches_read_span_exhaustively() {
        let alphabets: &[&[&str]] = &[
            &["a", "\n"],
            &["a", "b", "c", "\n"],
            &["a", "b", "\r\n"],
            &["a", "\r", "\n", "\r\n"],
            &["x", "y", "\n", "é", "🦀", "\r\n"],
        ];
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        let mut checked = 0usize;
        for alpha in alphabets {
            for _ in 0..3000 {
                let n = rng.below(14);
                let mut s = String::new();
                for _ in 0..n {
                    s.push_str(alpha[rng.below(alpha.len())]);
                }
                // Include `len + 1` / `len + 2`, which are past EOF (never char
                // boundaries) to exercise the out-of-bounds rejection.
                for off in 0..=s.len() + 2 {
                    if off <= s.len() && !s.is_char_boundary(off) {
                        continue;
                    }
                    for ctx in 0..=2 {
                        for &len in &[0usize, 1, 4] {
                            if check(&s, off, len, ctx) {
                                checked += 1;
                            }
                        }
                    }
                }
            }
        }
        assert!(checked > 100_000, "expected a broad sweep, only checked {checked}");
    }
}

#[cfg(test)]
mod scanner_tests {
    #![expect(
        clippy::cast_possible_truncation,
        reason = "deterministic fuzz inputs are tightly bounded"
    )]

    use super::*;

    /// Deterministic xorshift so any failure reproduces from a fixed seed.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        fn below(&mut self, n: usize) -> usize {
            (self.next() % n as u64) as usize
        }
    }

    /// Run one query against a (stateful) scanner and assert the result is
    /// identical to a fresh `SpanReader` — data, span, line, column, and
    /// `line_count` on success, `None` on failure. `history` is the
    /// queries already issued to this scanner, which its state (and therefore
    /// any failure) depends on.
    fn check(
        scanner: &mut SpanScanner<'_>,
        input: &[u8],
        span: (usize, usize),
        before: usize,
        after: usize,
        history: &[(usize, usize)],
    ) {
        let source_span = Span::sized(span.0 as u32, span.1 as u32);
        let expected =
            SpanReader::new(input, SpanRequest::new(source_span), ContextLines::new(before, after))
                .read();
        let got = scanner.read_span(source_span);
        let src = String::from_utf8_lossy(input);
        match (&expected, &got) {
            (Some(expected), Some(got)) => {
                let fields =
                    |c: &SpanContents<'_>| (*c.span(), c.line(), c.column(), c.line_count());
                assert_eq!(
                    (fields(expected), expected.data()),
                    (fields(got), got.data()),
                    "mismatch for src={src:?} span={span:?} before={before} after={after} \
                     history={history:?}"
                );
            }
            (None, None) => {}
            _ => panic!(
                "expected {expected:?}, got {got:?} for src={src:?} span={span:?} \
                 before={before} after={after} history={history:?}"
            ),
        }
    }

    /// Differentially fuzz `SpanScanner` against `SpanReader` with query
    /// *sequences* — the scanner's whole point is state carried between
    /// queries, so single-shot checks would miss its interesting bugs. Covers
    /// LF / CRLF / lone-CR and multibyte sources; spans of any alignment
    /// (byte offsets, not char boundaries — neither function cares) including
    /// past-EOF ones; queries in random order (which exercises the
    /// out-of-index fallbacks) and in the renderer's sorted-then-merge order.
    #[test]
    #[cfg_attr(
        miri,
        ignore = "equivalence fuzzer over safe, bounds-checked code — Miri finds no UB here \
                  and interprets it orders of magnitude slower; the scanner path is still \
                  exercised under Miri by the normal snapshot tests"
    )]
    fn scanner_matches_span_reader_exhaustively() {
        let alphabets: &[&[&str]] = &[
            &["a", "\n"],
            &["a", "b", "c", "\n"],
            &["a", "b", "\r\n"],
            &["a", "\r", "\n", "\r\n"],
            &["x", "y", "\n", "é", "🦀", "\r\n"],
        ];
        let mut rng = Rng(0xA076_1D64_78BD_642F);
        let mut checked = 0usize;
        for alpha in alphabets {
            for _ in 0..700 {
                let n = rng.below(16);
                let mut s = String::new();
                for _ in 0..n {
                    s.push_str(alpha[rng.below(alpha.len())]);
                }
                let input = s.as_bytes();
                for (before, after) in [(0, 0), (1, 1), (2, 2), (0, 2), (2, 0)] {
                    let mut spans: Vec<(usize, usize)> = std::iter::repeat_with(|| {
                        (rng.below(input.len() + 3), [0, 1, 2, 5][rng.below(4)])
                    })
                    .take(6)
                    .collect();

                    // Random order: queries may jump backwards past the
                    // index origin.
                    let mut scanner = SpanScanner::new(input, before, after);
                    for i in 0..spans.len() {
                        check(&mut scanner, input, spans[i], before, after, &spans[..i]);
                        checked += 1;
                    }

                    // The renderer's order: labels sorted by offset, then a
                    // merge attempt spanning from the first label to the
                    // furthest end.
                    spans.sort_unstable();
                    let first = spans[0].0;
                    let merged_end = spans.iter().map(|&(o, l)| o + l).max().unwrap();
                    spans.push((first, merged_end - first));
                    let mut scanner = SpanScanner::new(input, before, after);
                    for i in 0..spans.len() {
                        check(&mut scanner, input, spans[i], before, after, &spans[..i]);
                        checked += 1;
                    }
                }
            }
        }
        assert!(checked > 100_000, "expected a broad sweep, only checked {checked}");
    }

    /// The empty-source and just-past-EOF edge cases `SpanReader` special
    /// cases, issued through one scanner.
    #[test]
    fn zero_length_spans_at_eof() {
        let mut scanner = SpanScanner::new(b"", 0, 0);
        check(&mut scanner, b"", (0, 0), 0, 0, &[]);
        check(&mut scanner, b"", (1, 0), 0, 0, &[(0, 0)]);
        let mut scanner = SpanScanner::new(b"a", 1, 1);
        check(&mut scanner, b"a", (1, 0), 1, 1, &[]);
        check(&mut scanner, b"a", (2, 0), 1, 1, &[(1, 0)]);
    }
}
