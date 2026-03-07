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

    pub(super) fn push(&mut self, line: impl AsRef<str>) {
        if self.has_content {
            self.buf.push('\n');
        }
        self.buf.push_str(line.as_ref());
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

    /// Check whether content written after `from_byte` contains a blank line (`\n\n`).
    pub(super) fn has_blank_line_since(&self, from_byte: usize) -> bool {
        self.buf[from_byte..].contains("\n\n")
    }

    /// Count how many `\n` separators exist in content written after `from_byte`.
    pub(super) fn line_count_since(&self, from_byte: usize) -> usize {
        // Each \n before from_byte is a prior separator; count new ones.
        self.buf[from_byte..].bytes().filter(|&b| b == b'\n').count()
    }

    pub(super) fn into_string(self) -> String {
        self.buf
    }
}
