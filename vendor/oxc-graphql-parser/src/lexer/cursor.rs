use crate::Error;

/// Byte cursor over GraphQL source text.
#[derive(Debug)]
pub(crate) struct Cursor<'a> {
    pub(super) index: usize,
    pub(super) offset: usize,
    pub(super) source: &'a str,
    pub(super) bytes: &'a [u8],
    pub(super) next: usize,
    pub(crate) err: Option<Error>,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(input: &'a str) -> Cursor<'a> {
        Cursor { index: 0, offset: 0, source: input, bytes: input.as_bytes(), next: 0, err: None }
    }
}

impl<'a> Cursor<'a> {
    /// Current place (index) in the cursor.
    pub(crate) fn index(&self) -> usize {
        self.index
    }

    /// Consumes the remaining bytes of a name token and returns its full text.
    ///
    /// The first name byte is already consumed by `bump` in `advance`; this
    /// scans the rest of the name in a tight loop over the raw bytes, avoiding
    /// re-dispatching each byte through `advance`. It leaves the
    /// cursor in the exact position the per-byte path would: stopped before the
    /// terminator (mirroring `prev_str`), or at end of input with the
    /// EOF-adjacent index preserved for token-limit diagnostics (mirroring
    /// `current_str`).
    pub(super) fn consume_name(&mut self) -> &'a str {
        let len = self.bytes.len();
        let mut end = self.next;
        while end < len && super::is_name_continue(self.bytes[end]) {
            end += 1;
        }

        let slice = &self.source[self.index..end];
        self.index = if end == len && end > 0 { end - 1 } else { end };
        self.offset = end;
        self.next = end;
        slice
    }

    /// Returns the token text before the last consumed byte and rewinds to it.
    pub(crate) fn prev_str(&mut self) -> &'a str {
        let slice = &self.source[self.index..self.offset];

        self.index = self.offset;
        self.next = self.offset;

        slice
    }

    /// Returns the token text through the last consumed byte.
    pub(crate) fn current_str(&mut self) -> &'a str {
        let slice = &self.source[self.index..self.next];
        // Preserve the previous EOF-adjacent cursor position used by token-limit diagnostics.
        self.index =
            if self.next == self.source.len() && self.next > 0 { self.next - 1 } else { self.next };
        slice
    }

    /// Moves to the next byte.
    pub(crate) fn bump(&mut self) -> Option<u8> {
        if self.next == self.bytes.len() {
            return None;
        }

        let c = self.bytes[self.next];
        self.offset = self.next;
        self.next += 1;

        Some(c)
    }

    /// Consumes the next byte if it matches.
    pub(crate) fn eatc(&mut self, c: u8) -> bool {
        if self.next < self.bytes.len() && self.bytes[self.next] == c {
            self.offset = self.next;
            self.next += 1;
            return true;
        }

        false
    }

    /// Consumes the rest of the UTF-8 scalar at the current byte offset.
    pub(crate) fn consume_current_char(&mut self) -> char {
        let c = self.source[self.offset..].chars().next().unwrap();
        self.next = self.offset + c.len_utf8();
        c
    }

    /// Consumes a Unicode byte order mark at the current byte offset.
    pub(crate) fn eat_bom(&mut self) -> bool {
        const BOM: &[u8] = b"\xEF\xBB\xBF";

        if self.bytes[self.offset..].starts_with(BOM) {
            self.next = self.offset + BOM.len();
            return true;
        }

        false
    }

    /// Whether the next bytes are a Unicode byte order mark.
    pub(super) fn at_bom(&self) -> bool {
        self.bytes[self.next..].starts_with(b"\xEF\xBB\xBF")
    }

    /// Consumes the remaining bytes of a comment (the `#` is already consumed)
    /// and returns the end of its text.
    ///
    /// Scans to the next line terminator with `memchr` instead of re-dispatching
    /// each byte through `advance`. Leaves the cursor exactly where the per-byte path
    /// would: stopped before the terminator (mirroring `prev_str`), or at end
    /// of input with the EOF-adjacent index preserved for token-limit
    /// diagnostics (mirroring `current_str`).
    pub(super) fn seek_line_end(&mut self) -> usize {
        let end = match memchr::memchr2(b'\n', b'\r', &self.bytes[self.next..]) {
            Some(found) => {
                let end = self.next + found;
                self.index = end;
                end
            }
            None => {
                let end = self.bytes.len();
                // `end >= 1` because the leading `#` is already consumed.
                self.index = end - 1;
                end
            }
        };
        self.offset = end;
        self.next = end;
        end
    }

    /// Consumes the remaining bytes of a whitespace run and returns its text.
    ///
    /// The first whitespace unit is already consumed by `bump` in `advance`; this
    /// scans the rest of the run in a tight loop over the raw bytes (assimilated
    /// whitespace plus byte-order marks), avoiding re-dispatching each byte
    /// through `advance`. It leaves the cursor exactly where the
    /// per-byte path would: stopped before the terminator (mirroring `prev_str`),
    /// or at end of input with the EOF-adjacent index preserved for token-limit
    /// diagnostics (mirroring `current_str`).
    pub(super) fn consume_whitespace(&mut self) -> &'a str {
        const BOM: &[u8] = b"\xEF\xBB\xBF";
        let len = self.bytes.len();
        let mut end = self.next;
        while end < len {
            let byte = self.bytes[end];
            if super::is_whitespace_assimilated(byte) {
                end += 1;
            } else if byte == 0xEF && self.bytes[end..].starts_with(BOM) {
                end += BOM.len();
            } else {
                break;
            }
        }

        let slice = &self.source[self.index..end];
        self.index = if end == len && end > 0 { end - 1 } else { end };
        self.offset = end;
        self.next = end;
        slice
    }

    /// Drains the current token to the end of the source.
    pub(crate) fn drain(&mut self) -> &'a str {
        let start = self.index;
        self.index = self.source.len();
        self.next = self.source.len();

        self.source.get(start..).unwrap()
    }

    /// Add error object to the cursor.
    pub(crate) fn add_err(&mut self, err: Error) {
        self.err = Some(err)
    }
}
