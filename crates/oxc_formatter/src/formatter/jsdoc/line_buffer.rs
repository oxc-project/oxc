/// Append-only line buffer backed by a single `String`.
///
/// Reduces per-line heap allocations during serialization: all content goes into
/// one contiguous buffer separated by `\n`. Call `into_string()` at the end to
/// get the final `\n`-separated string with zero allocation.
pub(super) struct LineBuffer {
    buf: String,
    /// Whether at least one line has been pushed.
    has_content: bool,
}

impl LineBuffer {
    pub(super) fn new() -> Self {
        Self { buf: String::new(), has_content: false }
    }

    /// Push a line (or multiple lines if the string contains embedded `\n`).
    /// Each `\n` in the input creates a new line in the buffer.
    pub(super) fn push(&mut self, line: impl AsRef<str>) {
        let s = line.as_ref();
        if self.has_content {
            self.buf.push('\n');
        }
        self.buf.push_str(s);
        self.has_content = true;
    }

    pub(super) fn push_empty(&mut self) {
        self.push("");
    }

    /// Start a new line and return the buffer for direct writes.
    /// Callers **must** write content before calling `last_is_empty()` or
    /// `push_empty()`, since the separator `\n` is already appended.
    pub(super) fn begin_line(&mut self) -> &mut String {
        if self.has_content {
            self.buf.push('\n');
        }
        self.has_content = true;
        &mut self.buf
    }

    /// Check if the last pushed line was empty.
    pub(super) fn last_is_empty(&self) -> bool {
        // An empty last line means the buffer either:
        // - has exactly one empty line: has_content && buf.is_empty()
        // - ends with '\n' (separator before the empty last line)
        self.has_content && (self.buf.is_empty() || self.buf.ends_with('\n'))
    }

    pub(super) fn is_empty(&self) -> bool {
        !self.has_content
    }

    /// Current byte length of the buffer. Use as a snapshot marker
    /// to inspect content written between two points.
    pub(super) fn byte_len(&self) -> usize {
        self.buf.len()
    }

    /// Count how many `\n` separators exist in content written after `from_byte`.
    pub(super) fn line_count_since(&self, from_byte: usize) -> usize {
        // Each \n before from_byte is a prior separator; count new ones.
        self.buf[from_byte..].bytes().filter(|&b| b == b'\n').count()
    }

    /// Check if the last non-empty line ends a block-level element (list item
    /// or code block). Used to decide whether a trailing blank line is needed
    /// before the next tag.
    pub(super) fn last_line_is_block_end(&self) -> bool {
        // Walk backwards from end to find the last non-empty line.
        let last = self.buf.rsplit('\n').find(|l| !l.is_empty()).unwrap_or("");
        let trimmed = last.trim_start();
        // List item markers: `- `, `+ `, `* `, or `1. ` etc.
        if trimmed.starts_with("- ")
            || trimmed.starts_with("+ ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("```")
        {
            return true;
        }
        // Ordered list: digit(s) followed by `. `
        if let Some(rest) = trimmed.strip_prefix(|c: char| c.is_ascii_digit()) {
            let rest = rest.trim_start_matches(|c: char| c.is_ascii_digit());
            if rest.starts_with(". ") {
                return true;
            }
        }
        // Indented code block (4+ spaces) — already handled by checking if last
        // line content starts with 4 spaces
        if last.starts_with("    ") {
            return true;
        }
        false
    }

    pub(super) fn into_string(self) -> String {
        self.buf
    }
}
