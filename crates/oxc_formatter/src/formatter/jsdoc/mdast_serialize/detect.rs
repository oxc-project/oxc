/// Check if the text contains markdown constructs that require full AST parsing.
/// Returns `false` only for pure plain-text paragraphs that `wrap_plain_paragraphs()`
/// can handle directly (no lists, tables, code fences, headings, blockquotes, or
/// inline markdown like emphasis/links).
pub(super) fn needs_mdast_parsing(text: &str) -> bool {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        match bytes[i] {
            // Strikethrough, images, backslash escapes, HTML tags
            b'~' | b'\\' | b'<' => return true,
            // Emphasis/strong (*) and underscore (_): only trigger when they
            // could be markdown emphasis (adjacent to non-space), not when
            // used as arithmetic (`2 * 3`) or separators.
            // Also trigger for `* ` at line starts which could be list markers.
            b'*' | b'_' => {
                let next = if i + 1 < len { bytes[i + 1] } else { b' ' };
                let prev = if i > 0 { bytes[i - 1] } else { b' ' };
                // Emphasis: `*word` or `word*` (adjacent to non-space on at
                // least one side). `2 * 3` (spaces on both sides) is arithmetic.
                if !next.is_ascii_whitespace() || !prev.is_ascii_whitespace() {
                    return true;
                }
                // `* ` at line start: could be an unordered list marker
                if bytes[i] == b'*' && next == b' ' && (i == 0 || prev == b'\n') {
                    return true;
                }
            }
            // `[` — only trigger for markdown link/reference patterns, not bare
            // brackets from JavaScript code (e.g. `const [`, `][]`).
            b'[' => {
                // Footnote reference: `[^note]`
                if i + 1 < len && bytes[i + 1] == b'^' {
                    return true;
                }
                // Scan for closing `]` followed by `(` or `[` on the same line
                // to detect `[text](url)` or `[text][ref]` patterns.
                let mut j = i + 1;
                let mut has_content = false;
                while j < len && bytes[j] != b'\n' {
                    if bytes[j] == b']' {
                        if has_content
                            && j + 1 < len
                            && (bytes[j + 1] == b'(' || bytes[j + 1] == b'[')
                        {
                            return true;
                        }
                        break;
                    }
                    if !bytes[j].is_ascii_whitespace() {
                        has_content = true;
                    }
                    j += 1;
                }
            }
            // At line start (after optional leading spaces): detect block-level constructs.
            // We skip leading spaces to catch indented lists/code blocks.
            b' ' | b'#' | b'>' | b'-' | b'0'..=b'9' | b'|' | b'+'
                if i == 0 || bytes[i - 1] == b'\n' =>
            {
                // Check if this line starts a new block context (after blank line
                // or at text start). Lines that are paragraph continuations
                // (preceded by non-empty line) should not trigger block detection
                // for ambiguous markers like `-`, `+`, digits, `|`.
                let is_block_start = i == 0
                    || (i >= 2 && bytes[i - 1] == b'\n' && bytes[i - 2] == b'\n')
                    || (i >= 3
                        && bytes[i - 1] == b'\n'
                        && bytes[i - 2] == b' '
                        && bytes[i - 3] == b'\n');

                // Count leading spaces
                let mut spaces = 0;
                while i + spaces < len && bytes[i + spaces] == b' ' {
                    spaces += 1;
                }
                // 4+ leading spaces = indented code block (only at block start)
                if spaces >= 4 && is_block_start {
                    return true;
                }
                // Check trigger character after whitespace
                if i + spaces < len {
                    match bytes[i + spaces] {
                        // Headings and blockquotes are unambiguous — always trigger
                        b'#' | b'>' => return true,
                        // Digits: ordered lists (1. foo) or legacy markers (1- foo)
                        b'0'..=b'9' => {
                            // Only trigger if digits are followed by `. `, `) `, or `- `
                            // to avoid false positives from prose like "...and\n1. They"
                            let mut j = i + spaces;
                            while j < len && bytes[j].is_ascii_digit() {
                                j += 1;
                            }
                            if j < len && j + 1 < len && bytes[j + 1] == b' ' {
                                match bytes[j] {
                                    b'.' | b')' if is_block_start => return true,
                                    b'-' => return true, // legacy marker always
                                    _ => {}
                                }
                            }
                        }
                        b'|' => {
                            // Table detection: require pipe at start AND end of line
                            // (i.e., `| cell | cell |` pattern). Bare `|word|` in
                            // prose should not trigger.
                            let line_start = i + spaces;
                            if bytes[line_start] == b'|' {
                                // Find end of line
                                let mut line_end = line_start + 1;
                                while line_end < len && bytes[line_end] != b'\n' {
                                    line_end += 1;
                                }
                                // Check if line ends with `|` (after trimming spaces)
                                let mut end = line_end;
                                while end > line_start + 1 && bytes[end - 1].is_ascii_whitespace() {
                                    end -= 1;
                                }
                                if end > line_start + 1 && bytes[end - 1] == b'|' {
                                    return true;
                                }
                            }
                        }
                        // Unordered list markers: only at block start to avoid
                        // false positives from wrapped text like "min\n+ spacing"
                        b'-' | b'+' | b'*'
                            if is_block_start
                                && i + spaces + 1 < len
                                && bytes[i + spaces + 1] == b' ' =>
                        {
                            return true;
                        }
                        _ => {}
                    }
                }
            }
            // Code fences
            b'`' if i + 2 < len && bytes[i + 1] == b'`' && bytes[i + 2] == b'`' => {
                return true;
            }
            _ => {}
        }
        i += 1;
    }
    false
}
