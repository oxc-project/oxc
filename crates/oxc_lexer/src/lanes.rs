// Kernel lint policy — see the note in `pipeline/mod.rs`.
#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![allow(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::needless_range_loop,
    clippy::manual_range_contains,
    clippy::collapsible_if,
    clippy::collapsible_match
)]

#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
use core::arch::x86_64::*;

use crate::error::{Diagnostic, diag_code, diag_severity};
use crate::token::StringSpan;

#[derive(Default)]
pub struct Lanes {
    pub numbers: Vec<f64>,
    pub strings: Vec<StringSpan>,
    pub templates: Vec<StringSpan>,
    pub atoms: Vec<StringSpan>,
    pub regex_flags: Vec<u8>,
    pub cooked: Vec<u8>,
    pub comment_meta: Vec<u8>,
    pub comments: Vec<oxc_ast::ast::Comment>,
    /// Lexer diagnostics, pushed only on cold error paths; empty for valid input.
    pub diags: Vec<Diagnostic>,
    /// Byte ranges lexed content-blind (the `.tsx` type-argument skip): diagnostics landing in one are dropped at drain time.
    pub diag_suppress: Vec<(u32, u32)>,
    /// Positions of non-whitespace non-ASCII lead bytes, recorded by
    /// `misc_pre` before carve clears literal interiors. Resolved at drain:
    /// leads inside literal tokens are legal content and drop cheaply; only
    /// code-level survivors pay the identifier-char check. Empty for
    /// pure-ASCII input.
    pub unicode_leads: Vec<u32>,
    pub module: bool,
}

impl Lanes {
    pub fn clear(&mut self) {
        self.numbers.clear();
        self.strings.clear();
        self.templates.clear();
        self.atoms.clear();
        self.regex_flags.clear();
        self.cooked.clear();
        self.comment_meta.clear();
        self.comments.clear();
        self.diags.clear();
        self.diag_suppress.clear();
        self.unicode_leads.clear();
    }

    #[inline]
    pub fn push_regex_flags(&mut self, src: &[u8], fs: usize, fe: usize) {
        let mut f: u8 = 0;
        for (k, &c) in src[fs..fe].iter().enumerate() {
            let bit: u8 = match c {
                b'g' => 1,
                b'i' => 2,
                b'm' => 4,
                b's' => 8,
                b'u' => 16,
                b'y' => 32,
                b'd' => 64,
                b'v' => 128,
                _ => 0,
            };
            // Unknown or repeated flag: diagnostic. At most 8 flag chars.
            if bit == 0 {
                self.push_diag((fs + k) as u32, 1, crate::error::diag_code::INVALID_REGEXP_FLAG);
            } else if f & bit != 0 {
                self.push_diag((fs + k) as u32, 1, crate::error::diag_code::DUPLICATE_REGEXP_FLAG);
            }
            f |= bit;
        }
        self.regex_flags.push(f);
    }

    /// Record a diagnostic. Cold and non-inlined so the push stays out of
    /// the hot carve body.
    #[cold]
    #[inline(never)]
    pub fn push_diag(&mut self, off: u32, len: u32, code: u16) {
        self.diags.push(Diagnostic { off, len, code, severity: diag_severity::ERROR });
    }

    /// Line terminator in a string, with oxc_parser's exact span: opener
    /// through the first unescaped CR/LF (for CRLF only the CR). Escaped
    /// terminators are legal LineContinuations and are skipped the way the
    /// parser's escape reader skips them. LS/PS are legal string content.
    #[cold]
    #[inline(never)]
    pub fn push_line_terminator_in_string(&mut self, src: &[u8], s: usize, end: usize) {
        let mut i = s + 1; // past the opening quote
        let mut term_end = end; // fallback (unreachable: the scanner saw a terminator)
        while i < end {
            match src[i] {
                b'\\' => {
                    i += 1;
                    if i < end && src[i] == b'\r' {
                        i += 1;
                        if i < end && src[i] == b'\n' {
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
                b'\r' | b'\n' => {
                    term_end = i + 1;
                    break;
                }
                _ => i += 1,
            }
        }
        self.push_diag(s as u32, (term_end - s) as u32, diag_code::LINE_TERMINATOR_IN_STRING);
    }

    /// IdentifierStart or digit right after a numeric literal (`1.5n`,
    /// `3in`, `0b12`), with oxc_parser's `invalid_number_end` span: the char
    /// at `e2` plus the run of identifier-start chars after it.
    #[cold]
    #[inline(never)]
    pub fn push_num_end_diag(&mut self, src: &[u8], e2: usize, code: u16) {
        let end = ident_start_run_end(src, e2 + 1);
        self.push_diag(e2 as u32, (end - e2) as u32, code);
    }

    /// Non-ASCII byte right after a numeric literal: a Unicode
    /// IdentifierStart (`1π`) is the same invalid adjacency as the ASCII
    /// case; Unicode whitespace is legal and invalid UTF-8 emits nothing
    /// here.
    #[cold]
    #[inline(never)]
    pub fn push_num_end_diag_unicode(&mut self, src: &[u8], e2: usize) {
        let Some(ch) = decode_char_at(src, e2) else { return };
        if !oxc_syntax::identifier::is_identifier_start(ch) {
            return;
        }
        let end = ident_start_run_end(src, e2 + ch.len_utf8());
        self.push_diag(
            e2 as u32,
            (end - e2) as u32,
            crate::error::diag_code::INVALID_NUMERIC_LITERAL,
        );
    }

    #[inline]
    /// `EMIT`: report malformed escapes while cooking — strings only, since
    /// template escapes are legal when tagged (the parser owns that error).
    /// `CRLF`: normalize raw CRLF/CR to LF per the template TV rule
    /// (ECMA-262 12.9.6.1). Monomorphized out of copies that don't need them.
    fn cook<const EMIT: bool, const CRLF: bool>(
        &mut self,
        src: &[u8],
        bs: u32,
        be: u32,
    ) -> (StringSpan, bool) {
        let ss = self.cooked.len() as u32;
        if be <= bs {
            return (StringSpan::new(ss, ss & StringSpan::END_MASK, false), false);
        }
        let body_len = (be - bs) as usize;

        self.cooked.reserve(body_len.max(32) + 8);
        let base = self.cooked.as_mut_ptr();

        let needs_decode = if body_len < 32 {
            return self.cook_short_dispatch::<EMIT, CRLF>(src, bs, be, base, ss, body_len);
        } else if CRLF {
            span_has_bs_or_cr(src, bs as usize, be as usize)
        } else {
            span_has_bs(src, bs as usize, be as usize)
        };
        let (wi, lone, nes) = if needs_decode {
            unsafe { cook_decode::<EMIT, CRLF>(src.as_ptr(), bs, be, base, ss, &mut self.diags) }
        } else {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    src.as_ptr().add(bs as usize),
                    base.add(ss as usize),
                    body_len,
                );
            }
            (ss + (be - bs), false, false)
        };
        unsafe { self.cooked.set_len(wi as usize) };
        (StringSpan::new(ss, wi & StringSpan::END_MASK, lone), nes)
    }

    #[inline]
    fn cook_short_dispatch<const EMIT: bool, const CRLF: bool>(
        &mut self,
        src: &[u8],
        bs: u32,
        be: u32,
        base: *mut u8,
        ss: u32,
        body_len: usize,
    ) -> (StringSpan, bool) {
        let (wi, lone, nes) =
            cook_short::<EMIT, CRLF>(src, bs, be, base, ss, body_len, &mut self.diags);
        unsafe { self.cooked.set_len(wi as usize) };
        (StringSpan::new(ss, wi & StringSpan::END_MASK, lone), nes)
    }

    #[inline]
    pub fn push_string(&mut self, src: &[u8], bs: usize, be: usize) {
        let (sp, _) = self.cook::<true, false>(src, bs as u32, be as u32);
        self.strings.push(sp);
    }

    #[inline]
    pub fn push_template(&mut self, src: &[u8], bs: usize, be: usize) {
        // CRLF normalization is implemented and tested but wired off: the
        // extra `\r` needle costs ~2% on template-heavy sources, and the
        // parser facade cooks CR-bearing templates itself, so end-to-end
        // values are exact either way.
        let (sp, nes) = self.cook::<false, false>(src, bs as u32, be as u32);
        // A NotEscapeSequence means cooked = None per the grammar; legality
        // depends on tagged-ness (parser context), so the span carries a
        // marker instead of a diagnostic.
        self.templates.push(if nes { sp.with_cooked_invalid() } else { sp });
    }

    /// Push a string value verbatim. JSX attribute strings have no backslash
    /// escapes; the value is the raw slice between the quotes.
    #[inline]
    pub fn push_string_raw(&mut self, src: &[u8], bs: usize, be: usize) {
        let ss = self.cooked.len() as u32;
        if be <= bs {
            self.strings.push(StringSpan::new(ss, ss & StringSpan::END_MASK, false));
            return;
        }
        let body_len = be - bs;
        self.cooked.reserve(body_len + 8);
        unsafe {
            let base = self.cooked.as_mut_ptr();
            core::ptr::copy_nonoverlapping(src.as_ptr().add(bs), base.add(ss as usize), body_len);
            self.cooked.set_len(ss as usize + body_len);
        }
        let we = ss + body_len as u32;
        self.strings.push(StringSpan::new(ss, we & StringSpan::END_MASK, false));
    }

    #[inline]
    pub fn push_atom(&mut self, src: &[u8], bs: usize, be: usize) {
        // Escaped-identifier atoms are the one funnel every code-level
        // identifier escape passes through, so validation rides here.
        self.validate_ident_escapes(src, bs, be);
        let (sp, _) = self.cook::<false, false>(src, bs as u32, be as u32);
        self.atoms.push(sp);
    }

    /// A well-formed escape decoding to a non-identifier char (`var \u0020 = 1`):
    /// oxc_parser reports the decoded char with an empty span after the escape.
    /// We emit `(end, 0)`; the bridge recognizes the len-0 shape and decodes the
    /// escape text backward to recover the char.
    #[cold]
    #[inline(never)]
    fn check_escaped_ident_char(&mut self, value: u32, at_start: bool, end: usize) {
        let Some(ch) = char::from_u32(value) else { return }; // surrogates handled by caller
        let ok = if at_start {
            oxc_syntax::identifier::is_identifier_start(ch)
        } else {
            oxc_syntax::identifier::is_identifier_part(ch)
        };
        if !ok {
            self.push_diag(end as u32, 0, crate::error::diag_code::UNEXPECTED_CHARACTER);
        }
    }

    /// Validate every unicode escape in an identifier atom, mirroring
    /// oxc_parser's consumption exactly: spans start after the backslash and
    /// end where the parser's scanner stopped; surrogate pairs are invalid in
    /// identifiers (one diag over both escapes); a failed pair-low rewinds so
    /// the second escape is validated on its own.
    #[cold]
    #[inline(never)]
    fn validate_ident_escapes(&mut self, src: &[u8], bs: usize, be: usize) {
        use crate::error::diag_code as D;
        fn hex4(src: &[u8], mut k: usize, be: usize) -> (Option<u32>, usize) {
            let mut v = 0u32;
            for _ in 0..4 {
                if k >= be {
                    return (None, k);
                }
                let Some(h) = hexd(src[k]) else { return (None, k) };
                v = (v << 4) | u32::from(h);
                k += 1;
            }
            (Some(v), k)
        }
        let mut i = bs;
        while i < be {
            if src[i] != b'\\' {
                i += 1;
                continue;
            }
            // `is_identifier_start` applies when the escape begins the atom
            // (for private names `bs` is already past the `#`).
            let at_start = i == bs;
            let start = i + 1;
            if start >= be || src[start] != b'u' {
                // Unreachable via scan_ident_esc, but mirror the parser
                // defensively: consume one char, span = it.
                self.push_diag(start as u32, u32::from(start < be), D::INVALID_IDENTIFIER_ESCAPE);
                i = start + 1;
                continue;
            }
            let mut k = start + 1; // after `u`
            if k < be && src[k] == b'{' {
                k += 1;
                let mut value: u32 = 0;
                let mut any = false;
                let mut over = false;
                while k < be {
                    let Some(h) = hexd(src[k]) else { break };
                    value = (value << 4) | u32::from(h);
                    any = true;
                    k += 1;
                    if value > 0x0010_FFFF {
                        over = true; // parser bails right after this digit
                        break;
                    }
                }
                if !over && any && k < be && src[k] == b'}' {
                    k += 1; // `}` consumed only on the well-formed path
                    if (0xD800..=0xDFFF).contains(&value) {
                        // lone surrogate via braces: invalid in identifiers
                        self.push_diag(
                            start as u32,
                            (k - start) as u32,
                            D::INVALID_IDENTIFIER_ESCAPE,
                        );
                    } else {
                        self.check_escaped_ident_char(value, at_start, k);
                    }
                } else {
                    // overflow / no digits / missing `}`: end = parser stop
                    self.push_diag(start as u32, (k - start) as u32, D::INVALID_IDENTIFIER_ESCAPE);
                }
                i = k;
                continue;
            }
            let (h1, e1) = hex4(src, k, be);
            let Some(high) = h1 else {
                self.push_diag(start as u32, (e1 - start) as u32, D::INVALID_IDENTIFIER_ESCAPE);
                i = e1;
                continue;
            };
            k = e1;
            if !(0xD800..=0xDFFF).contains(&high) {
                // well-formed BMP code point: must still be an identifier char
                self.check_escaped_ident_char(high, at_start, k);
                i = k;
                continue;
            }
            if (0xD800..=0xDBFF).contains(&high)
                && k + 1 < be
                && src[k] == b'\\'
                && src[k + 1] == b'u'
            {
                let (l1, e2) = hex4(src, k + 2, be);
                if let Some(low) = l1 {
                    if (0xDC00..=0xDFFF).contains(&low) {
                        // A well-formed pair is still invalid in identifiers:
                        // one diag over both escapes.
                        self.push_diag(
                            start as u32,
                            (e2 - start) as u32,
                            D::INVALID_IDENTIFIER_ESCAPE,
                        );
                        i = e2;
                        continue;
                    }
                }
                // Not a valid low: fall through — the parser rewinds and
                // reports the first escape alone.
            }
            self.push_diag(start as u32, (k - start) as u32, D::INVALID_IDENTIFIER_ESCAPE);
            i = k;
        }
    }

    #[inline]
    pub fn push_number(&mut self, src: &[u8], s: usize, e: usize) {
        self.numbers.push(parse_number(src, s, e));
    }

    #[inline]
    pub fn push_number_swar(&mut self, src: &[u8], s: usize, e: usize) {
        let len = e - s;
        if len.wrapping_sub(1) <= 7 && !(len > 1 && src[s] == b'0') {
            let keep = KEEP[len];
            let raw = unsafe { core::ptr::read_unaligned(src.as_ptr().add(s) as *const u64) };
            let w = raw & keep;
            let f0 = 0xF0F0_F0F0_F0F0_F0F0 & keep;
            let three = 0x3030_3030_3030_3030 & keep;
            let digits =
                (w & f0) == three && (w.wrapping_add(0x0606_0606_0606_0606 & keep) & f0) == three;
            if digits {
                let mut d = (w.wrapping_sub(three)) << ((8 - len) * 8);
                d = ((d & 0x0f00_0f00_0f00_0f00) >> 8) + (d & 0x000f_000f_000f_000f) * 10;
                d = ((d & 0x00ff_0000_00ff_0000) >> 16) + (d & 0x0000_00ff_0000_00ff) * 100;
                d = ((d & 0x0000_ffff_0000_0000) >> 32) + (d & 0x0000_0000_0000_ffff) * 10000;
                self.numbers.push(d as f64);
                return;
            }
        }
        // Only non-SWAR numbers (floats, radix-prefixed, bigint, separators, leading zeros, long) reach the validation walk.
        let code = validate_number(src, s, e);
        if code != crate::error::diag_code::OK {
            self.push_diag(s as u32, (e - s) as u32, code);
        }
        self.numbers.push(parse_number(src, s, e));
    }

    #[inline]
    pub fn push_comment_record(
        &mut self,
        src: &[u8],
        sl: usize,
        start: u32,
        end: u32,
        blk: bool,
        meta: u8,
    ) {
        use oxc_ast::ast::{Comment, CommentKind};
        let kind = if !blk {
            CommentKind::Line
        } else if meta & crate::comment_meta::META_MULTILINE != 0 {
            CommentKind::MultiLineBlock
        } else {
            CommentKind::SingleLineBlock
        };
        let mut c = Comment::new(start, end.min(sl as u32), kind);
        c.content = crate::comment_meta::content_from_ordinal(meta);

        let s_ = (start as usize).min(sl);
        let mut q = s_;
        while q > 0 && is_lex_ws(src[q - 1]) {
            q -= 1;
        }
        let pre = q == 0 || src[q..s_].iter().any(|&b| b == b'\n' || b == b'\r');
        c.set_preceded_by_newline(pre);

        if !blk {
            c.set_followed_by_newline(true);
        } else {
            let mut e = (end as usize).min(sl);
            let mut post = false;
            while e < sl && is_lex_ws(src[e]) {
                if src[e] == b'\n' || src[e] == b'\r' {
                    post = true;
                    break;
                }
                e += 1;
            }
            c.set_followed_by_newline(post);
        }
        self.comments.push(c);
    }
}

#[inline]
fn is_lex_ws(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

static KEEP: [u64; 9] = [
    0x0000_0000_0000_0000,
    0x0000_0000_0000_00ff,
    0x0000_0000_0000_ffff,
    0x0000_0000_00ff_ffff,
    0x0000_0000_ffff_ffff,
    0x0000_00ff_ffff_ffff,
    0x0000_ffff_ffff_ffff,
    0x00ff_ffff_ffff_ffff,
    0xffff_ffff_ffff_ffff,
];

#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
#[inline]
fn cook_short<const EMIT: bool, const CRLF: bool>(
    src: &[u8],
    bs: u32,
    be: u32,
    base: *mut u8,
    ss: u32,
    body_len: usize,
    diags: &mut Vec<Diagnostic>,
) -> (u32, bool, bool) {
    let v = unsafe { _mm256_loadu_si256(src.as_ptr().add(bs as usize) as *const __m256i) };
    let mut hit = unsafe { _mm256_cmpeq_epi8(v, _mm256_set1_epi8(b'\\' as i8)) };
    if CRLF {
        // Templates also decode when a raw CR needs LF-normalizing.
        hit = unsafe { _mm256_or_si256(hit, _mm256_cmpeq_epi8(v, _mm256_set1_epi8(b'\r' as i8))) };
    }
    let bs_mask = unsafe { _mm256_movemask_epi8(hit) as u32 } & ((1u32 << body_len) - 1);
    if bs_mask == 0 {
        unsafe { _mm256_storeu_si256(base.add(ss as usize) as *mut __m256i, v) };
        (ss + (be - bs), false, false)
    } else {
        unsafe { cook_decode::<EMIT, CRLF>(src.as_ptr(), bs, be, base, ss, diags) }
    }
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
#[inline]
fn cook_short<const EMIT: bool, const CRLF: bool>(
    src: &[u8],
    bs: u32,
    be: u32,
    base: *mut u8,
    ss: u32,
    body_len: usize,
    diags: &mut Vec<Diagnostic>,
) -> (u32, bool, bool) {
    let body = &src[bs as usize..be as usize];
    let plain = if CRLF {
        memchr::memchr2(b'\\', b'\r', body).is_none()
    } else {
        memchr::memchr(b'\\', body).is_none()
    };
    if plain {
        unsafe {
            core::ptr::copy_nonoverlapping(
                src.as_ptr().add(bs as usize),
                base.add(ss as usize),
                body_len,
            );
        }
        (ss + (be - bs), false, false)
    } else {
        unsafe { cook_decode::<EMIT, CRLF>(src.as_ptr(), bs, be, base, ss, diags) }
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
#[inline]
fn span_has_bs(src: &[u8], bs: usize, be: usize) -> bool {
    let mut i = bs;
    while i + 32 <= be {
        let v = unsafe { _mm256_loadu_si256(src.as_ptr().add(i) as *const __m256i) };
        let m =
            unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi8(v, _mm256_set1_epi8(b'\\' as i8))) };
        if m != 0 {
            return true;
        }
        i += 32;
    }
    while i < be {
        if src[i] == b'\\' {
            return true;
        }
        i += 1;
    }
    false
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
#[inline]
fn span_has_bs(src: &[u8], bs: usize, be: usize) -> bool {
    memchr::memchr(b'\\', &src[bs..be]).is_some()
}

/// Template variant of [`span_has_bs`]: a raw CR also forces the decode path.
#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
#[inline]
fn span_has_bs_or_cr(src: &[u8], bs: usize, be: usize) -> bool {
    let mut i = bs;
    while i + 32 <= be {
        let v = unsafe { _mm256_loadu_si256(src.as_ptr().add(i) as *const __m256i) };
        let m = unsafe {
            _mm256_movemask_epi8(_mm256_or_si256(
                _mm256_cmpeq_epi8(v, _mm256_set1_epi8(b'\\' as i8)),
                _mm256_cmpeq_epi8(v, _mm256_set1_epi8(b'\r' as i8)),
            ))
        };
        if m != 0 {
            return true;
        }
        i += 32;
    }
    while i < be {
        if src[i] == b'\\' || src[i] == b'\r' {
            return true;
        }
        i += 1;
    }
    false
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
#[inline]
fn span_has_bs_or_cr(src: &[u8], bs: usize, be: usize) -> bool {
    memchr::memchr2(b'\\', b'\r', &src[bs..be]).is_some()
}

/// Decode the single `char` at byte `i`, or `None` if the bytes there are
/// not valid UTF-8.
pub(crate) fn decode_char_at(s: &[u8], i: usize) -> Option<char> {
    let end = (i + 4).min(s.len());
    let slice = s.get(i..end)?;
    match core::str::from_utf8(slice) {
        Ok(t) => t.chars().next(),
        // a multi-byte char cut at `end` still has a decodable valid prefix
        Err(e) if e.valid_up_to() > 0 => {
            core::str::from_utf8(&slice[..e.valid_up_to()]).ok()?.chars().next()
        }
        Err(_) => None,
    }
}

/// End of the run of IdentifierStart chars beginning at `i` — the tail of
/// oxc_parser's `invalid_number_end` span (a digit ends the run).
fn ident_start_run_end(s: &[u8], mut i: usize) -> usize {
    while i < s.len() {
        let b = s[i];
        if b < 0x80 {
            if !(b.is_ascii_alphabetic() || b == b'_' || b == b'$') {
                break;
            }
            i += 1;
        } else {
            let Some(c) = decode_char_at(s, i) else { break };
            if !oxc_syntax::identifier::is_identifier_start(c) {
                break;
            }
            i += c.len_utf8();
        }
    }
    i
}

#[inline(always)]
fn hexd(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn parse_number(src: &[u8], s: usize, e: usize) -> f64 {
    let mut i = s;
    let mut radix: u64 = 0;
    if e - s >= 2 && src[s] == b'0' {
        let d = src[s + 1];
        radix = match d {
            b'x' | b'X' => 16,
            b'b' | b'B' => 2,
            b'o' | b'O' => 8,
            _ => 0,
        };
        if radix != 0 {
            i = s + 2;
        }
    }
    if radix != 0 {
        let mut v: u64 = 0;
        while i < e {
            let c = src[i];
            i += 1;
            if c == b'_' {
                continue;
            }
            if c == b'n' {
                break;
            }
            match hexd(c) {
                Some(h) => v = v.wrapping_mul(radix).wrapping_add(h as u64),
                None => break,
            }
        }
        return v as f64;
    }

    let mut flt = false;
    let mut dl = 0usize;
    let mut v: u64 = 0;
    for k in s..e {
        let c = src[k];
        if c == b'.' || c == b'e' || c == b'E' {
            flt = true;
        } else if c.is_ascii_digit() {
            dl += 1;
            v = v.wrapping_mul(10).wrapping_add((c - b'0') as u64);
        }
    }
    if !flt && dl <= 19 {
        return v as f64;
    }

    let mut buf = [0u8; 512];
    let mut bl = 0usize;
    let mut k = s;
    while k < e && bl < buf.len() - 1 {
        let c = src[k];
        k += 1;
        if c == b'_' || c == b'n' {
            continue;
        }
        buf[bl] = c;
        bl += 1;
    }
    core::str::from_utf8(&buf[..bl]).ok().and_then(|st| st.parse::<f64>().ok()).unwrap_or(0.0)
}

#[inline]
fn radix_digit(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// First error code for the numeric literal `src[s..e]`, or `diag_code::OK`.
/// Conservative by design: it must never flag a valid number, so anything
/// ambiguous returns OK. Covers what survives as one token span — separator
/// misplacement, empty radix (`0x`), legacy-octal-like decimals with a
/// separator/bigint suffix/bad exponent. Shapes the scanner pre-splits
/// (`0b12`, `1.5n`, `3in`) are detected at the adjacency in coalesce, not
/// here. Detection only; `parse_number` still produces a lenient value.
#[inline(never)]
fn validate_number(src: &[u8], s: usize, e: usize) -> u16 {
    use crate::error::diag_code as D;
    let bytes = &src[s..e];
    let Some((&last, head)) = bytes.split_last() else { return D::OK };
    let is_bigint = last == b'n';
    let body = if is_bigint { head } else { bytes };
    if body.is_empty() {
        return D::OK;
    }

    // Radix-prefixed integer: separator placement and the empty-radix case.
    // An out-of-range digit cannot appear (the scanner stops before it), so
    // that arm is a safety net.
    if body.len() >= 2 && body[0] == b'0' {
        let radix = match body[1] | 0x20 {
            b'x' => 16u8,
            b'o' => 8,
            b'b' => 2,
            _ => 0,
        };
        if radix != 0 {
            let mut prev_us = true; // a '_' immediately after the prefix is invalid
            let mut any = false;
            for &c in &body[2..] {
                if c == b'_' {
                    if prev_us {
                        return D::INVALID_NUMERIC_SEPARATOR;
                    }
                    prev_us = true;
                } else if matches!(radix_digit(c), Some(v) if v < radix) {
                    any = true;
                    prev_us = false;
                } else {
                    return D::INVALID_NUMERIC_LITERAL;
                }
            }
            if any && prev_us {
                return D::INVALID_NUMERIC_SEPARATOR; // trailing '_'
            }
            if !any {
                return D::INVALID_NUMERIC_LITERAL; // empty radix, e.g. `0x`
            }
            return D::OK;
        }

        // Legacy-octal-like decimal (leading `0` + digit/`_`): oxc_parser
        // consumes only `[0-9]` here — no separators, no bigint suffix — and
        // accepts an exponent only as lowercase `e` after an `8`/`9` flipped
        // the run to NonOctalDecimal (`08e1` valid; `00e1` and `08E1` not).
        // Bare `00`/`08` are valid sloppy-mode Annex B, and `.` never flags.
        if matches!(body[1], b'0'..=b'9' | b'_') {
            if bytes.contains(&b'_') {
                return D::INVALID_NUMERIC_SEPARATOR;
            }
            if is_bigint {
                return D::INVALID_BIGINT;
            }
            let run = bytes.iter().take_while(|c| c.is_ascii_digit()).count();
            match bytes.get(run) {
                // `08e1`: valid NonOctalDecimal exponent; its digits are
                // still checked below (`08e` stays flagged).
                Some(&b'e') if bytes[..run].iter().any(|&c| c >= b'8') => {}
                Some(&b'e' | &b'E') => return D::INVALID_NUMERIC_LITERAL,
                _ => {}
            }
            // fall through: the valid shapes still get the generic checks
        }
    }

    // Every '_' must sit between two digits.
    let mut prev_digit = false;
    let mut exp_at: Option<usize> = None;
    for (i, &c) in body.iter().enumerate() {
        if c == b'_' {
            let next_digit = matches!(body.get(i + 1), Some(&d) if d.is_ascii_digit());
            if !prev_digit || !next_digit {
                return D::INVALID_NUMERIC_SEPARATOR;
            }
            prev_digit = false;
        } else {
            prev_digit = c.is_ascii_digit();
            if (c | 0x20) == b'e' && exp_at.is_none() {
                exp_at = Some(i); // radix bodies returned above, so this is the marker
            }
        }
    }
    // Empty exponent (`1e`, `1e+`, `.5e`): the marker and optional sign were
    // consumed but no digits followed — never true for a valid literal.
    if let Some(ep) = exp_at {
        let mut k = ep + 1;
        if matches!(body.get(k), Some(&s) if s == b'+' || s == b'-') {
            k += 1;
        }
        if !matches!(body.get(k), Some(d) if d.is_ascii_digit()) {
            return D::INVALID_NUMERIC_LITERAL;
        }
    }
    D::OK
}

#[inline]
unsafe fn push_cp(out: *mut u8, mut wi: u32, cp: u32, lone: &mut bool) -> u32 {
    if cp <= 0x7F {
        *out.add(wi as usize) = cp as u8;
        wi += 1;
    } else if cp <= 0x7FF {
        *out.add(wi as usize) = 0xC0 | ((cp >> 6) as u8);
        *out.add((wi + 1) as usize) = 0x80 | ((cp & 0x3F) as u8);
        wi += 2;
    } else if (0xD800..=0xDFFF).contains(&cp) {
        *lone = true;
        *out.add(wi as usize) = 0xE0 | ((cp >> 12) as u8);
        *out.add((wi + 1) as usize) = 0x80 | (((cp >> 6) & 0x3F) as u8);
        *out.add((wi + 2) as usize) = 0x80 | ((cp & 0x3F) as u8);
        wi += 3;
    } else if cp <= 0xFFFF {
        *out.add(wi as usize) = 0xE0 | ((cp >> 12) as u8);
        *out.add((wi + 1) as usize) = 0x80 | (((cp >> 6) & 0x3F) as u8);
        *out.add((wi + 2) as usize) = 0x80 | ((cp & 0x3F) as u8);
        wi += 3;
    } else if cp <= 0x10FFFF {
        *out.add(wi as usize) = 0xF0 | ((cp >> 18) as u8);
        *out.add((wi + 1) as usize) = 0x80 | (((cp >> 12) & 0x3F) as u8);
        *out.add((wi + 2) as usize) = 0x80 | (((cp >> 6) & 0x3F) as u8);
        *out.add((wi + 3) as usize) = 0x80 | ((cp & 0x3F) as u8);
        wi += 4;
    } else {
        *out.add(wi as usize) = 0xEF;
        *out.add((wi + 1) as usize) = 0xBF;
        *out.add((wi + 2) as usize) = 0xBD;
        wi += 3;
    }
    wi
}

#[inline]
unsafe fn emit_oct(
    src: *const u8,
    out: *mut u8,
    mut wi: u32,
    mut i: u32,
    be: u32,
    mut val: u32,
) -> (u32, u32) {
    let mut cnt = 1;
    while cnt < 3 && i < be {
        let dd = *src.add(i as usize);
        if !(b'0'..=b'7').contains(&dd) {
            break;
        }
        let nx = val * 8 + (dd - b'0') as u32;
        if nx > 0xFF {
            break;
        }
        val = nx;
        i += 1;
        cnt += 1;
    }
    if val < 0x80 {
        *out.add(wi as usize) = val as u8;
        wi += 1;
    } else {
        *out.add(wi as usize) = 0xC0 | ((val >> 6) as u8);
        *out.add((wi + 1) as usize) = 0x80 | ((val & 0x3F) as u8);
        wi += 2;
    }
    (wi, i)
}

/// Bad string escape, with oxc_parser's span: the backslash to just past the
/// last byte its escape scanner consumed.
#[cold]
#[inline(never)]
fn push_escape_diag(diags: &mut Vec<Diagnostic>, off: u32, end: u32) {
    diags.push(Diagnostic {
        off,
        len: end - off,
        code: diag_code::INVALID_UNICODE_ESCAPE,
        severity: diag_severity::ERROR,
    });
}

unsafe fn cook_decode<const EMIT: bool, const CRLF: bool>(
    src: *const u8,
    bs: u32,
    be: u32,
    out: *mut u8,
    ss: u32,
    diags: &mut Vec<Diagnostic>,
) -> (u32, bool, bool) {
    let mut wi = ss;
    let mut i = bs;
    let mut lone = false;
    // NotEscapeSequence per the template grammar (bad \u or \x, octal, \8,
    // \9): push_template marks the span cooked-invalid from it, the stand-in
    // for oxc_parser's cooked = None.
    let mut nes = false;
    while i < be {
        let b = *src.add(i as usize);
        if b != b'\\' {
            // template TV: raw CRLF / CR normalize to LF
            if CRLF && b == b'\r' {
                *out.add(wi as usize) = b'\n';
                wi += 1;
                i += 1;
                if i < be && *src.add(i as usize) == b'\n' {
                    i += 1;
                }
                continue;
            }
            *out.add(wi as usize) = b;
            wi += 1;
            i += 1;
            continue;
        }
        if i + 1 >= be {
            *out.add(wi as usize) = b;
            wi += 1;
            i += 1;
            continue;
        }
        let nb = *src.add((i + 1) as usize);
        i += 2;
        match nb {
            b'b' => {
                *out.add(wi as usize) = 0x08;
                wi += 1;
            }
            b'f' => {
                *out.add(wi as usize) = 0x0C;
                wi += 1;
            }
            b'n' => {
                *out.add(wi as usize) = b'\n';
                wi += 1;
            }
            b'r' => {
                *out.add(wi as usize) = b'\r';
                wi += 1;
            }
            b't' => {
                *out.add(wi as usize) = b'\t';
                wi += 1;
            }
            b'v' => {
                *out.add(wi as usize) = 0x0B;
                wi += 1;
            }
            b'0' => {
                // spellchecker:disable-next-line
                let nd = i < be && (*src.add(i as usize)).is_ascii_digit();
                // spellchecker:disable-next-line
                if !nd {
                    *out.add(wi as usize) = 0;
                    wi += 1;
                } else {
                    nes = true; // octal after backslash-0: NotEscapeSequence in templates
                    let (nwi, ni) = emit_oct(src, out, wi, i, be, 0);
                    wi = nwi;
                    i = ni;
                }
            }
            b'\\' | b'\'' | b'"' | b'`' => {
                *out.add(wi as usize) = nb;
                wi += 1;
            }
            b'\n' => {}
            b'\r' => {
                if i < be && *src.add(i as usize) == b'\n' {
                    i += 1;
                }
            }
            b'x' => {
                // The parser's span runs to just past the leading valid hex.
                if i + 2 <= be {
                    let a = hexd(*src.add(i as usize));
                    let c = hexd(*src.add((i + 1) as usize));
                    if let (Some(a), Some(c)) = (a, c) {
                        *out.add(wi as usize) = (a << 4) | c;
                        wi += 1;
                        i += 2;
                    } else {
                        *out.add(wi as usize) = b'x';
                        wi += 1;
                        nes = true;
                        if EMIT {
                            push_escape_diag(diags, i - 2, i + u32::from(a.is_some()));
                        }
                    }
                } else {
                    *out.add(wi as usize) = b'x';
                    wi += 1;
                    nes = true;
                    if EMIT {
                        let k = i < be && hexd(*src.add(i as usize)).is_some();
                        push_escape_diag(diags, i - 2, i + u32::from(k));
                    }
                }
            }
            b'u' => {
                let esc = i - 2; // backslash offset, for code-7 spans
                if i < be && *src.add(i as usize) == b'{' {
                    let mut k = i + 1;
                    let mut cp: u32 = 0;
                    let mut any = false;
                    // The parser bails on the digit that pushes the value
                    // past 0x10FFFF. Latch that point in-loop: `cp` wraps, so
                    // a post-loop range check would miss inputs like
                    // `\u{100000041}`.
                    let mut over_end: u32 = 0;
                    while k < be && *src.add(k as usize) != b'}' {
                        if let Some(h) = hexd(*src.add(k as usize)) {
                            cp = (cp << 4) | (h as u32);
                            any = true;
                            k += 1;
                            if cp > 0x0010_FFFF && over_end == 0 {
                                over_end = k;
                            }
                        } else {
                            break;
                        }
                    }
                    if k < be && *src.add(k as usize) == b'}' && any {
                        wi = push_cp(out, wi, cp, &mut lone);
                        i = k + 1;
                        if over_end != 0 {
                            nes = true;
                        }
                        if EMIT && over_end != 0 {
                            // out of range but well-formed (`\u{110000}`):
                            // cooked value (U+FFFD) unchanged, diagnostic only
                            push_escape_diag(diags, esc, over_end);
                        }
                    } else {
                        *out.add(wi as usize) = b'u';
                        wi += 1;
                        i = k;
                        nes = true;
                        if EMIT {
                            push_escape_diag(diags, esc, if over_end != 0 { over_end } else { k });
                        }
                    }
                } else if i + 4 <= be {
                    let a = hexd(*src.add(i as usize));
                    let b2 = hexd(*src.add((i + 1) as usize));
                    let c = hexd(*src.add((i + 2) as usize));
                    let d = hexd(*src.add((i + 3) as usize));
                    if let (Some(a), Some(b2), Some(c), Some(d)) = (a, b2, c, d) {
                        let cp = ((a as u32) << 12)
                            | ((b2 as u32) << 8)
                            | ((c as u32) << 4)
                            | (d as u32);
                        i += 4;
                        // Adjacent 4-digit escapes forming a surrogate pair combine
                        // into one code point (`\uD834\uDF06` is U+1D306), matching
                        // oxc_parser's SurrogatePair rule; braced escapes never pair.
                        let mut cp = cp;
                        if (0xD800..=0xDBFF).contains(&cp)
                            && i + 6 <= be
                            && *src.add(i as usize) == b'\\'
                            && *src.add((i + 1) as usize) == b'u'
                        {
                            let l0 = hexd(*src.add((i + 2) as usize));
                            let l1 = hexd(*src.add((i + 3) as usize));
                            let l2 = hexd(*src.add((i + 4) as usize));
                            let l3 = hexd(*src.add((i + 5) as usize));
                            if let (Some(l0), Some(l1), Some(l2), Some(l3)) = (l0, l1, l2, l3) {
                                let lo = ((l0 as u32) << 12)
                                    | ((l1 as u32) << 8)
                                    | ((l2 as u32) << 4)
                                    | (l3 as u32);
                                if (0xDC00..=0xDFFF).contains(&lo) {
                                    cp = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00);
                                    i += 6;
                                }
                            }
                        }
                        wi = push_cp(out, wi, cp, &mut lone);
                    } else {
                        *out.add(wi as usize) = b'u';
                        wi += 1;
                        nes = true;
                        if EMIT {
                            // span covers the leading valid hex run
                            let j = u32::from(a.is_some())
                                + u32::from(a.is_some() && b2.is_some())
                                + u32::from(a.is_some() && b2.is_some() && c.is_some());
                            push_escape_diag(diags, esc, i + j);
                        }
                    }
                } else {
                    *out.add(wi as usize) = b'u';
                    wi += 1;
                    nes = true;
                    if EMIT {
                        let mut k = i;
                        while k < be && hexd(*src.add(k as usize)).is_some() {
                            k += 1;
                        }
                        push_escape_diag(diags, esc, k);
                    }
                }
            }
            b'1'..=b'7' => {
                nes = true; // octal escape: NotEscapeSequence in templates
                let (nwi, ni) = emit_oct(src, out, wi, i, be, (nb - b'0') as u32);
                wi = nwi;
                i = ni;
            }
            b'8' | b'9' => {
                // NonOctalDecimalEscape: cooks to the digit, valid in sloppy
                // strings, NotEscapeSequence in templates.
                nes = true;
                *out.add(wi as usize) = nb;
                wi += 1;
            }
            _ => {
                // Escaped LS/PS is a LineContinuation and cooks to nothing.
                // Every other char is an identity escape: its bytes flow
                // through (the lead here, continuations via the plain copy).
                if nb == 0xE2
                    && i + 2 <= be
                    && *src.add(i as usize) == 0x80
                    && matches!(*src.add((i + 1) as usize), 0xA8 | 0xA9)
                {
                    i += 2;
                } else {
                    *out.add(wi as usize) = nb;
                    wi += 1;
                }
            }
        }
    }
    (wi, lone, nes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cook(body: &[u8]) -> (Vec<u8>, bool) {
        let mut padded = body.to_vec();
        padded.extend_from_slice(&[0u8; 32]);
        let mut out = vec![0u8; body.len() + 32];
        let mut diags = Vec::new();
        let (wi, lone, _) = unsafe {
            cook_decode::<true, false>(
                padded.as_ptr(),
                0,
                body.len() as u32,
                out.as_mut_ptr(),
                0,
                &mut diags,
            )
        };
        out.truncate(wi as usize);
        (out, lone)
    }

    /// The "detection only" guarantee: EMIT=true must cook byte-identically
    /// to EMIT=false on every malformed escape (fallback chars preserved).
    #[test]
    fn emit_does_not_change_cooked_bytes() {
        for body in [
            br"\u{110000}".as_slice(),
            br"\uZZZZ",
            br"\xGG",
            br"\u{}",
            br"\u{12",
            br"\u{xyz",
            br"\x4G",
            br"\u004",
        ] {
            let mut padded = body.to_vec();
            padded.extend_from_slice(&[0u8; 32]);
            let mut out_t = vec![0u8; body.len() + 32];
            let mut out_f = vec![0u8; body.len() + 32];
            let mut diags = Vec::new();
            let (wt, _, _) = unsafe {
                cook_decode::<true, false>(
                    padded.as_ptr(),
                    0,
                    body.len() as u32,
                    out_t.as_mut_ptr(),
                    0,
                    &mut diags,
                )
            };
            let (wf, _, _) = unsafe {
                cook_decode::<false, false>(
                    padded.as_ptr(),
                    0,
                    body.len() as u32,
                    out_f.as_mut_ptr(),
                    0,
                    &mut Vec::new(),
                )
            };
            assert_eq!(wt, wf);
            assert_eq!(out_t[..wt as usize], out_f[..wf as usize]);
            assert!(
                !diags.is_empty(),
                "expected a diagnostic for {:?}",
                String::from_utf8_lossy(body)
            );
        }
    }

    #[test]
    fn cook_decode_escapes() {
        assert_eq!(cook(br"\n").0, b"\n");
        assert_eq!(cook(br"\b\f\t\r\v").0, &[0x08, 0x0C, b'\t', b'\r', 0x0B]);
        assert_eq!(cook(br"\x41").0, b"A");
        assert_eq!(cook(br"\u0041").0, b"A");
        assert_eq!(cook(br"\u{41}").0, b"A");
        assert_eq!(cook(br"\u{1F600}").0, "\u{1F600}".as_bytes());
        assert_eq!(cook(br"\101").0, b"A");
        assert_eq!(cook(br"\0").0, b"\0");
        assert_eq!(cook(br#"\\\"\'"#).0, br#"\"'"#);
        assert_eq!(cook(br"\u{xyz").0, b"uxyz");
        assert_eq!(cook(br"\u{}").0, b"u}");
        assert_eq!(cook(br"\u{12").0, b"u");
    }

    #[test]
    fn cook_decode_lone_surrogate() {
        let (out, lone) = cook(br"\uD83D");
        assert!(lone, "lone-surrogate flag must be set");
        assert_eq!(out, [0xED, 0xA0, 0xBD]);
    }

    /// Adjacent 4-digit escapes forming a surrogate pair combine into one
    /// code point (oxc_parser's SurrogatePair rule); the braced form does not.
    #[test]
    fn cook_decode_surrogate_pairs() {
        let (out, lone) = cook(br"\uD834\uDF06");
        assert!(!lone, "a combined pair is not a lone surrogate");
        assert_eq!(out, "\u{1D306}".as_bytes());

        // A high escape followed by a non-low stays lone; the next escape
        // decodes independently.
        let (out, lone) = cook(br"\uD834\u0041");
        assert!(lone);
        assert_eq!(out, [0xED, 0xA0, 0xB4, b'A']);

        // Braced escapes never pair (parser parity).
        let (out, lone) = cook(br"\u{D834}\u{DF06}");
        assert!(lone, "braced surrogates stay lone");
        assert_eq!(out, [0xED, 0xA0, 0xB4, 0xED, 0xBC, 0x86]);
    }

    /// `\` + LS/PS is a LineContinuation: cooks to nothing (like `\<LF>`).
    #[test]
    fn cook_decode_ls_ps_line_continuation() {
        let mut body = Vec::new();
        body.extend_from_slice(b"a\\");
        body.extend_from_slice("\u{2028}".as_bytes());
        body.extend_from_slice(b"b\\");
        body.extend_from_slice("\u{2029}".as_bytes());
        body.extend_from_slice(b"c");
        assert_eq!(cook(&body).0, b"abc");
        // a raw LS/PS is content, not a continuation — it must survive
        let mut raw = Vec::new();
        raw.extend_from_slice(b"a");
        raw.extend_from_slice("\u{2028}".as_bytes());
        raw.extend_from_slice(b"b");
        assert_eq!(cook(&raw).0, raw);
    }

    /// Template TV: raw <CR><LF> and <CR> normalize to <LF>; escape-produced
    /// `\r` survives; the string monomorph (CRLF=false) never normalizes.
    #[test]
    fn cook_decode_template_cr_normalization() {
        fn cook_tpl(body: &[u8]) -> Vec<u8> {
            let mut padded = body.to_vec();
            padded.extend_from_slice(&[0u8; 32]);
            let mut out = vec![0u8; body.len() + 32];
            let (wi, _, _) = unsafe {
                cook_decode::<false, true>(
                    padded.as_ptr(),
                    0,
                    body.len() as u32,
                    out.as_mut_ptr(),
                    0,
                    &mut Vec::new(),
                )
            };
            out.truncate(wi as usize);
            out
        }
        assert_eq!(cook_tpl(b"a\r\nb"), b"a\nb");
        assert_eq!(cook_tpl(b"a\rb"), b"a\nb");
        assert_eq!(cook_tpl(b"a\r\r\nb"), b"a\n\nb");
        // Escape-produced CR is content, not a line ending.
        assert_eq!(cook_tpl(br"a\rb"), b"a\rb");
        // `\<CR><LF>` line continuation still cooks to nothing.
        assert_eq!(cook_tpl(b"a\\\r\nb"), b"ab");
        // The string monomorph is untouched by design.
        assert_eq!(cook(b"a\r\nb").0, b"a\r\nb");
    }
}
