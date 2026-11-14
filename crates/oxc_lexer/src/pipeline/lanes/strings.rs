use std::ptr;

use oxc_simd::{Avx2, BASELINE, Cpu, DynCpu, Neon, Vec128, Vec256, cpu_multiversion, cpu_version};
use oxc_syntax::identifier::{is_identifier_part, is_identifier_start};

use crate::{
    error::{DiagCode, DiagSeverity, Diagnostic},
    token::StringSpan,
};

use crate::pipeline::{
    bytes::{hex_digit, is_ls_ps_tail},
    chunk::mm32,
};

use super::Lanes;

impl Lanes {
    /// Line terminator in a string, with oxc_parser's exact span: opener
    /// through the first unescaped CR/LF (for CRLF only the CR). Escaped
    /// terminators are legal LineContinuations and are skipped the way the
    /// parser's escape reader skips them. LS/PS are legal string content.
    ///
    /// As it's cold code, it's compiled only once, not separately for each CPU feature set.
    /// `#[inline(never)]` stops it being inlined into the separate copies of its callers.
    #[cpu_version(Generic)]
    #[cold]
    #[inline(never)]
    pub(crate) fn push_line_terminator_in_string(&mut self, src: &[u8], s: usize, end: usize) {
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
        self.push_diag(s as u32, (term_end - s) as u32, DiagCode::LineTerminatorInString);
    }

    /// `EMIT`: report malformed escapes while cooking - strings only, since
    /// template escapes are legal when tagged (the parser owns that error).
    /// `CRLF`: normalize raw CRLF/CR to LF per the template TV rule
    /// (ECMA-262 12.9.6.1). Monomorphized out of copies that don't need them.
    #[cpu_multiversion]
    #[inline]
    fn cook<const EMIT: bool, const CRLF: bool>(
        &mut self,
        src: &[u8],
        bs: u32,
        be: u32,
        cpu: impl Cpu,
    ) -> (StringSpan, bool) {
        let ss = self.cooked.len() as u32;
        if be <= bs {
            return (StringSpan::new(ss, ss & StringSpan::END_MASK, false, cpu), false);
        }
        let body_len = (be - bs) as usize;

        self.cooked.reserve(body_len.max(32) + 8);
        let base = self.cooked.as_mut_ptr();

        let needs_decode = if body_len < 32 {
            return self.cook_short_dispatch::<EMIT, CRLF>(src, bs, be, base, ss, body_len, cpu);
        } else if CRLF {
            span_has_bs_or_cr(src, bs as usize, be as usize, cpu)
        } else {
            span_has_bs(src, bs as usize, be as usize, cpu)
        };
        let (wi, lone, nes) = if needs_decode {
            unsafe {
                cook_decode::<EMIT, CRLF>(src.as_ptr(), bs, be, base, ss, &mut self.diags, cpu)
            }
        } else {
            unsafe {
                ptr::copy_nonoverlapping(
                    src.as_ptr().add(bs as usize),
                    base.add(ss as usize),
                    body_len,
                );
            }
            (ss + (be - bs), false, false)
        };
        unsafe { self.cooked.set_len(wi as usize) };
        (StringSpan::new(ss, wi & StringSpan::END_MASK, lone, cpu), nes)
    }

    #[cpu_multiversion]
    #[inline(always)]
    fn cook_short_dispatch<const EMIT: bool, const CRLF: bool>(
        &mut self,
        src: &[u8],
        bs: u32,
        be: u32,
        base: *mut u8,
        ss: u32,
        body_len: usize,
        cpu: impl Cpu,
    ) -> (StringSpan, bool) {
        let (wi, lone, nes) =
            cook_short::<EMIT, CRLF>(src, bs, be, base, ss, body_len, &mut self.diags, cpu);
        unsafe { self.cooked.set_len(wi as usize) };
        (StringSpan::new(ss, wi & StringSpan::END_MASK, lone, cpu), nes)
    }

    #[cpu_multiversion]
    #[inline(always)]
    pub(crate) fn push_string(&mut self, src: &[u8], bs: usize, be: usize, cpu: impl Cpu) {
        let (sp, _) = self.cook::<true, false>(src, bs as u32, be as u32, cpu);
        self.strings.push(sp);
    }

    #[cpu_multiversion]
    #[inline(always)]
    pub(crate) fn push_template(&mut self, src: &[u8], bs: usize, be: usize, cpu: impl Cpu) {
        let (sp, nes) = self.cook::<false, true>(src, bs as u32, be as u32, cpu);
        // A NotEscapeSequence means cooked = None per the grammar; legality
        // depends on tagged-ness (parser context), so the span carries a
        // marker instead of a diagnostic.
        self.templates.push(if nes { sp.with_cooked_invalid(cpu) } else { sp });
    }

    /// Push a string value verbatim. JSX attribute strings have no backslash
    /// escapes; the value is the raw slice between the quotes.
    #[cpu_multiversion]
    #[inline]
    pub(crate) fn push_string_raw(&mut self, src: &[u8], bs: usize, be: usize, cpu: impl Cpu) {
        let ss = self.cooked.len() as u32;
        if be <= bs {
            self.strings.push(StringSpan::new(ss, ss & StringSpan::END_MASK, false, cpu));
            return;
        }
        let body_len = be - bs;
        self.cooked.reserve(body_len + 8);
        unsafe {
            let base = self.cooked.as_mut_ptr();
            ptr::copy_nonoverlapping(src.as_ptr().add(bs), base.add(ss as usize), body_len);
            self.cooked.set_len(ss as usize + body_len);
        }
        let we = ss + body_len as u32;
        self.strings.push(StringSpan::new(ss, we & StringSpan::END_MASK, false, cpu));
    }

    #[cpu_multiversion]
    #[inline(always)]
    pub(crate) fn push_atom(&mut self, src: &[u8], bs: usize, be: usize, cpu: impl Cpu) {
        // Escaped-identifier atoms are the one funnel every code-level
        // identifier escape passes through, so validation rides here.
        self.validate_ident_escapes(src, bs, be);
        let (sp, _) = self.cook::<false, false>(src, bs as u32, be as u32, cpu);
        self.atoms.push(sp);
    }

    /// A well-formed escape decoding to a non-identifier char (`var \u0020 = 1`):
    /// oxc_parser reports the decoded char with an empty span after the escape.
    /// We emit `(end, 0)`; the bridge recognizes the len-0 shape and decodes the
    /// escape text backward to recover the char.
    #[cpu_version(Generic)]
    #[cold]
    #[inline(never)]
    fn check_escaped_ident_char(&mut self, value: u32, at_start: bool, end: usize) {
        let Some(ch) = char::from_u32(value) else { return }; // surrogates handled by caller
        let ok = if at_start { is_identifier_start(ch) } else { is_identifier_part(ch) };
        if !ok {
            self.push_diag(end as u32, 0, DiagCode::UnexpectedCharacter);
        }
    }

    /// Validate every unicode escape in an identifier atom, mirroring
    /// oxc_parser's consumption exactly: spans start after the backslash and
    /// end where the parser's scanner stopped; surrogate pairs are invalid in
    /// identifiers (one diag over both escapes); a failed pair-low rewinds so
    /// the second escape is validated on its own.
    ///
    /// As it's cold code, it's compiled only once, not separately for each CPU feature set.
    /// `#[inline(never)]` stops it being inlined into the separate copies of its callers.
    #[cpu_version(Generic)]
    #[cold]
    #[inline(never)]
    fn validate_ident_escapes(&mut self, src: &[u8], bs: usize, be: usize) {
        #[cpu_version(Generic)]
        fn hex4(src: &[u8], mut k: usize, be: usize) -> (Option<u32>, usize) {
            let cpu = BASELINE;

            let mut v = 0u32;
            for _ in 0..4 {
                if k >= be {
                    return (None, k);
                }
                let Some(h) = hex_digit(src[k], cpu) else { return (None, k) };
                v = (v << 4) | u32::from(h);
                k += 1;
            }
            (Some(v), k)
        }

        let cpu = BASELINE;

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
                self.push_diag(
                    start as u32,
                    u32::from(start < be),
                    DiagCode::InvalidIdentifierEscape,
                );
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
                    let Some(h) = hex_digit(src[k], cpu) else { break };
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
                            DiagCode::InvalidIdentifierEscape,
                        );
                    } else {
                        self.check_escaped_ident_char(value, at_start, k);
                    }
                } else {
                    // overflow / no digits / missing `}`: end = parser stop
                    self.push_diag(
                        start as u32,
                        (k - start) as u32,
                        DiagCode::InvalidIdentifierEscape,
                    );
                }
                i = k;
                continue;
            }
            let (h1, e1) = hex4(src, k, be);
            let Some(high) = h1 else {
                self.push_diag(
                    start as u32,
                    (e1 - start) as u32,
                    DiagCode::InvalidIdentifierEscape,
                );
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
                let (low, low_end) = hex4(src, k + 2, be);
                if let Some(low) = low
                    && (0xDC00..=0xDFFF).contains(&low)
                {
                    // A well-formed pair is still invalid in identifiers:
                    // one diag over both escapes.
                    self.push_diag(
                        start as u32,
                        (low_end - start) as u32,
                        DiagCode::InvalidIdentifierEscape,
                    );
                    i = low_end;
                    continue;
                }
                // Not a valid low: fall through - the parser rewinds and
                // reports the first escape alone.
            }
            self.push_diag(start as u32, (k - start) as u32, DiagCode::InvalidIdentifierEscape);
            i = k;
        }
    }
}

#[cpu_multiversion]
#[inline(always)]
fn cook_short<const EMIT: bool, const CRLF: bool>(
    src: &[u8],
    bs: u32,
    be: u32,
    base: *mut u8,
    ss: u32,
    body_len: usize,
    diags: &mut Vec<Diagnostic>,
    cpu: impl Cpu,
) -> (u32, bool, bool) {
    match DynCpu::from(cpu) {
        DynCpu::Avx2(cpu) => {
            cook_short_avx2::<EMIT, CRLF>(src, bs, be, base, ss, body_len, diags, cpu)
        }
        DynCpu::Neon(cpu) => {
            cook_short_neon::<EMIT, CRLF>(src, bs, be, base, ss, body_len, diags, cpu)
        }
        DynCpu::Generic(_) => {
            cook_short_generic::<EMIT, CRLF>(src, bs, be, base, ss, body_len, diags)
        }
    }
}

/// [`cook_short`] for Neon.
#[cpu_version(Neon)]
#[inline]
fn cook_short_neon<const EMIT: bool, const CRLF: bool>(
    src: &[u8],
    bs: u32,
    be: u32,
    base: *mut u8,
    ss: u32,
    body_len: usize,
    diags: &mut Vec<Diagnostic>,
    cpu: Neon,
) -> (u32, bool, bool) {
    let p = unsafe { src.as_ptr().add(bs as usize) };
    let (v0, v1) =
        unsafe { (Vec128::load_unaligned(p, cpu), Vec128::load_unaligned(p.add(16), cpu)) };
    let (mut hit0, mut hit1) = (v0.eq_u8(b'\\'), v1.eq_u8(b'\\'));
    if CRLF {
        // Templates also decode when a raw CR needs LF-normalizing.
        hit0 |= v0.eq_u8(b'\r');
        hit1 |= v1.eq_u8(b'\r');
    }
    if mm32(hit0, hit1) & ((1u32 << body_len) - 1) == 0 {
        unsafe {
            v0.store_unaligned(base.add(ss as usize));
            v1.store_unaligned(base.add(ss as usize + 16));
        }
        (ss + (be - bs), false, false)
    } else {
        unsafe { cook_decode::<EMIT, CRLF>(src.as_ptr(), bs, be, base, ss, diags, cpu) }
    }
}

/// [`cook_short`] for AVX2.
#[cpu_version(Avx2)]
#[inline]
fn cook_short_avx2<const EMIT: bool, const CRLF: bool>(
    src: &[u8],
    bs: u32,
    be: u32,
    base: *mut u8,
    ss: u32,
    body_len: usize,
    diags: &mut Vec<Diagnostic>,
    cpu: Avx2,
) -> (u32, bool, bool) {
    let v = unsafe { Vec256::load_unaligned(src.as_ptr().add(bs as usize), cpu) };
    let mut hit = v.eq_u8(b'\\');
    if CRLF {
        // Templates also decode when a raw CR needs LF-normalizing.
        hit |= v.eq_u8(b'\r');
    }
    let bs_mask = hit.movemask() & ((1u32 << body_len) - 1);
    if bs_mask == 0 {
        unsafe { v.store_unaligned(base.add(ss as usize)) };
        (ss + (be - bs), false, false)
    } else {
        unsafe { cook_decode::<EMIT, CRLF>(src.as_ptr(), bs, be, base, ss, diags, cpu) }
    }
}

/// [`cook_short`] for all other CPUs.
#[cpu_version(Generic)]
#[inline]
fn cook_short_generic<const EMIT: bool, const CRLF: bool>(
    src: &[u8],
    bs: u32,
    be: u32,
    base: *mut u8,
    ss: u32,
    body_len: usize,
    diags: &mut Vec<Diagnostic>,
) -> (u32, bool, bool) {
    let cpu = BASELINE;

    let body = &src[bs as usize..be as usize];
    let plain = if CRLF {
        memchr::memchr2(b'\\', b'\r', body).is_none()
    } else {
        memchr::memchr(b'\\', body).is_none()
    };
    if plain {
        unsafe {
            ptr::copy_nonoverlapping(
                src.as_ptr().add(bs as usize),
                base.add(ss as usize),
                body_len,
            );
        }
        (ss + (be - bs), false, false)
    } else {
        unsafe { cook_decode::<EMIT, CRLF>(src.as_ptr(), bs, be, base, ss, diags, cpu) }
    }
}

#[cpu_multiversion]
#[inline(always)]
fn span_has_bs(src: &[u8], bs: usize, be: usize, cpu: impl Cpu) -> bool {
    match DynCpu::from(cpu) {
        DynCpu::Avx2(cpu) => span_has_bs_avx2(src, bs, be, cpu),
        DynCpu::Neon(cpu) => span_has_neon::<false>(src, bs, be, cpu),
        DynCpu::Generic(_) => span_has_bs_generic(src, bs, be),
    }
}

/// [`span_has_bs`] for AVX2.
#[cpu_version(Avx2)]
#[inline]
fn span_has_bs_avx2(src: &[u8], bs: usize, be: usize, cpu: Avx2) -> bool {
    let mut i = bs;
    while i + 32 <= be {
        let v = unsafe { Vec256::load_unaligned(src.as_ptr().add(i), cpu) };
        let m = v.eq_u8(b'\\').movemask();
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

/// [`span_has_bs`] for all other CPUs.
#[cpu_version(Generic)]
#[inline]
fn span_has_bs_generic(src: &[u8], bs: usize, be: usize) -> bool {
    memchr::memchr(b'\\', &src[bs..be]).is_some()
}

/// Template variant of [`span_has_bs`]: a raw CR also forces the decode path.
#[cpu_multiversion]
#[inline(always)]
fn span_has_bs_or_cr(src: &[u8], bs: usize, be: usize, cpu: impl Cpu) -> bool {
    match DynCpu::from(cpu) {
        DynCpu::Avx2(cpu) => span_has_bs_or_cr_avx2(src, bs, be, cpu),
        DynCpu::Neon(cpu) => span_has_neon::<true>(src, bs, be, cpu),
        DynCpu::Generic(_) => span_has_bs_or_cr_generic(src, bs, be),
    }
}

/// [`span_has_bs`] for Neon, or [`span_has_bs_or_cr`] if CR. Reads up to 15 bytes into the padding.
#[cpu_version(Neon)]
#[inline]
fn span_has_neon<const CR: bool>(src: &[u8], bs: usize, be: usize, cpu: Neon) -> bool {
    let hits = |i: usize| {
        let v = unsafe { Vec128::load_unaligned(src.as_ptr().add(i), cpu) };
        (if CR { v.eq_u8(b'\\') | v.eq_u8(b'\r') } else { v.eq_u8(b'\\') }).nibble_mask()
    };
    let mut i = bs;
    while i + 16 <= be {
        if hits(i) != 0 {
            return true;
        }
        i += 16;
    }
    i < be && hits(i) & ((1u64 << (4 * (be - i))) - 1) != 0
}

/// [`span_has_bs_or_cr`] for AVX2.
#[cpu_version(Avx2)]
#[inline]
fn span_has_bs_or_cr_avx2(src: &[u8], bs: usize, be: usize, cpu: Avx2) -> bool {
    let mut i = bs;
    while i + 32 <= be {
        let v = unsafe { Vec256::load_unaligned(src.as_ptr().add(i), cpu) };
        let m = (v.eq_u8(b'\\') | v.eq_u8(b'\r')).movemask();
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

/// [`span_has_bs_or_cr`] for all other CPUs.
#[cpu_version(Generic)]
#[inline]
fn span_has_bs_or_cr_generic(src: &[u8], bs: usize, be: usize) -> bool {
    memchr::memchr2(b'\\', b'\r', &src[bs..be]).is_some()
}

#[cpu_multiversion]
#[inline]
unsafe fn push_cp(out: *mut u8, mut wi: u32, cp: u32, lone: &mut bool, _cpu: impl Cpu) -> u32 {
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

#[cpu_multiversion]
#[inline]
unsafe fn emit_oct(
    src: *const u8,
    out: *mut u8,
    mut wi: u32,
    mut i: u32,
    be: u32,
    mut val: u32,
    _cpu: impl Cpu,
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
///
/// As it's cold code, it's compiled only once, not separately for each CPU feature set.
/// `#[inline(never)]` stops it being inlined into the separate copies of its callers.
#[cpu_version(Generic)]
#[cold]
#[inline(never)]
fn push_escape_diag(diags: &mut Vec<Diagnostic>, off: u32, end: u32) {
    diags.push(Diagnostic {
        off,
        len: end - off,
        code: DiagCode::InvalidUnicodeEscape,
        severity: DiagSeverity::Error,
    });
}

#[cpu_multiversion]
unsafe fn cook_decode<const EMIT: bool, const CRLF: bool>(
    src: *const u8,
    bs: u32,
    be: u32,
    out: *mut u8,
    ss: u32,
    diags: &mut Vec<Diagnostic>,
    cpu: impl Cpu,
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
                let nd = i < be && (*src.add(i as usize)).is_ascii_digit();
                if !nd {
                    *out.add(wi as usize) = 0;
                    wi += 1;
                } else {
                    nes = true; // octal after backslash-0: NotEscapeSequence in templates
                    let (nwi, ni) = emit_oct(src, out, wi, i, be, 0, cpu);
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
                    let a = hex_digit(*src.add(i as usize), cpu);
                    let c = hex_digit(*src.add((i + 1) as usize), cpu);
                    if let (Some(a), Some(c)) = (a, c) {
                        let v = (u32::from(a) << 4) | u32::from(c);
                        if v < 0x80 {
                            *out.add(wi as usize) = v as u8;
                            wi += 1;
                        } else {
                            *out.add(wi as usize) = 0xC0 | ((v >> 6) as u8);
                            *out.add((wi + 1) as usize) = 0x80 | ((v & 0x3F) as u8);
                            wi += 2;
                        }
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
                        let k = i < be && hex_digit(*src.add(i as usize), cpu).is_some();
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
                        if let Some(h) = hex_digit(*src.add(k as usize), cpu) {
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
                        wi = push_cp(out, wi, cp, &mut lone, cpu);
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
                    let a = hex_digit(*src.add(i as usize), cpu);
                    let b2 = hex_digit(*src.add((i + 1) as usize), cpu);
                    let c = hex_digit(*src.add((i + 2) as usize), cpu);
                    let d = hex_digit(*src.add((i + 3) as usize), cpu);
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
                            let l0 = hex_digit(*src.add((i + 2) as usize), cpu);
                            let l1 = hex_digit(*src.add((i + 3) as usize), cpu);
                            let l2 = hex_digit(*src.add((i + 4) as usize), cpu);
                            let l3 = hex_digit(*src.add((i + 5) as usize), cpu);
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
                        wi = push_cp(out, wi, cp, &mut lone, cpu);
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
                        while k < be && hex_digit(*src.add(k as usize), cpu).is_some() {
                            k += 1;
                        }
                        push_escape_diag(diags, esc, k);
                    }
                }
            }
            b'1'..=b'7' => {
                nes = true; // octal escape: NotEscapeSequence in templates
                let (nwi, ni) = emit_oct(src, out, wi, i, be, (nb - b'0') as u32, cpu);
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
                    && is_ls_ps_tail(*src.add(i as usize), *src.add((i + 1) as usize), cpu)
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
    use oxc_simd::test_cpus;

    use super::*;

    /// Push template `body` to a new [`Lanes`] with CPU feature set `cpu`,
    /// and return its cooked bytes.
    #[cpu_multiversion]
    fn push_tpl(body: &[u8], cpu: impl Cpu) -> Vec<u8> {
        let mut padded = body.to_vec();
        padded.extend_from_slice(&[0u8; 64]);
        let mut l = Lanes::default();
        l.push_template(&padded, 0, body.len(), cpu);
        let sp = l.templates[0];
        let s = (sp.start & StringSpan::START_MASK) as usize;
        let e = (sp.end_and_flags & StringSpan::END_MASK) as usize;
        l.cooked[s..e].to_vec()
    }

    #[test_cpus]
    fn push_template_normalizes_raw_cr(cpu: impl Cpu) {
        assert_eq!(push_tpl(b"a\r\nb", cpu), b"a\nb");
        assert_eq!(push_tpl(b"a\rb", cpu), b"a\nb");
        assert_eq!(push_tpl(br"a\r\nb\`c", cpu), b"a\r\nb`c");
        assert_eq!(push_tpl(b"a\r\nb\\`c", cpu), b"a\nb`c");
    }

    #[test_cpus]
    fn push_template_keeps_escaped_cr_lf(cpu: impl Cpu) {
        assert_eq!(push_tpl(br"\r\n", cpu), b"\r\n");
        assert_eq!(push_tpl(br"\r", cpu), b"\r");
        assert_eq!(push_tpl(b"\r\n", cpu), b"\n");
    }

    /// Cook string `body` with [`cook_decode`] and CPU feature set `cpu`.
    /// Returns the cooked bytes, and whether they contain a lone surrogate.
    #[cpu_multiversion]
    fn cook(body: &[u8], cpu: impl Cpu) -> (Vec<u8>, bool) {
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
                cpu,
            )
        };
        out.truncate(wi as usize);
        (out, lone)
    }

    /// The "detection only" guarantee: EMIT=true must cook byte-identically
    /// to EMIT=false on every malformed escape (fallback chars preserved).
    #[test_cpus]
    fn emit_does_not_change_cooked_bytes(cpu: impl Cpu) {
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
                    cpu,
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
                    cpu,
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

    #[test_cpus]
    fn cook_decode_escapes(cpu: impl Cpu) {
        assert_eq!(cook(br"\n", cpu).0, b"\n");
        assert_eq!(cook(br"\b\f\t\r\v", cpu).0, &[0x08, 0x0C, b'\t', b'\r', 0x0B]);
        assert_eq!(cook(br"\x41", cpu).0, b"A");
        assert_eq!(cook(br"\u0041", cpu).0, b"A");
        assert_eq!(cook(br"\u{41}", cpu).0, b"A");
        assert_eq!(cook(br"\u{1F600}", cpu).0, "\u{1F600}".as_bytes());
        assert_eq!(cook(br"\101", cpu).0, b"A");
        assert_eq!(cook(br"\0", cpu).0, b"\0");
        assert_eq!(cook(br#"\\\"\'"#, cpu).0, br#"\"'"#);
        assert_eq!(cook(br"\u{xyz", cpu).0, b"uxyz");
        assert_eq!(cook(br"\u{}", cpu).0, b"u}");
        assert_eq!(cook(br"\u{12", cpu).0, b"u");
    }

    #[test_cpus]
    fn cook_hex_escape_is_a_code_point(cpu: impl Cpu) {
        assert_eq!(
            cook(br"\x41\x7f\x80\xa0\xe9\xff", cpu).0,
            "A\u{7f}\u{80}\u{a0}\u{e9}\u{ff}".as_bytes()
        );
        assert_eq!(
            cook(br"\x41\x7f\x80\xa0\xe9\xff", cpu).0,
            &[0x41, 0x7F, 0xC2, 0x80, 0xC2, 0xA0, 0xC3, 0xA9, 0xC3, 0xBF]
        );
        assert_eq!(cook(br"\x00\x01\x1f", cpu).0, &[0x00, 0x01, 0x1F]);
        for cp in 0u32..=0xFF {
            let src = format!("\\x{cp:02x}");
            let want = char::from_u32(cp).unwrap().to_string();
            assert_eq!(cook(src.as_bytes(), cpu).0, want.as_bytes(), "\\x{cp:02x}");
        }
        assert!(std::str::from_utf8(&cook(br"\xff", cpu).0).is_ok());
    }

    #[test_cpus]
    fn cook_escape_forms_agree(cpu: impl Cpu) {
        for cp in 0u32..=0xFF {
            let hex = cook(format!("\\x{cp:02x}").as_bytes(), cpu).0;
            let uni = cook(format!("\\u{cp:04x}").as_bytes(), cpu).0;
            assert_eq!(hex, uni, "\\x vs \\u for U+{cp:04X}");
            if cp <= 0o377 {
                let oct = cook(format!("\\{cp:o}").as_bytes(), cpu).0;
                assert_eq!(hex, oct, "\\x vs octal for U+{cp:04X}");
            }
        }
    }

    #[test_cpus]
    fn cook_decode_lone_surrogate(cpu: impl Cpu) {
        let (out, lone) = cook(br"\uD83D", cpu);
        assert!(lone, "lone-surrogate flag must be set");
        assert_eq!(out, [0xED, 0xA0, 0xBD]);
    }

    /// Adjacent 4-digit escapes forming a surrogate pair combine into one
    /// code point (oxc_parser's SurrogatePair rule); the braced form does not.
    #[test_cpus]
    fn cook_decode_surrogate_pairs(cpu: impl Cpu) {
        let (out, lone) = cook(br"\uD834\uDF06", cpu);
        assert!(!lone, "a combined pair is not a lone surrogate");
        assert_eq!(out, "\u{1D306}".as_bytes());

        // A high escape followed by a non-low stays lone; the next escape
        // decodes independently.
        let (out, lone) = cook(br"\uD834\u0041", cpu);
        assert!(lone);
        assert_eq!(out, [0xED, 0xA0, 0xB4, b'A']);

        // Braced escapes never pair (parser parity).
        let (out, lone) = cook(br"\u{D834}\u{DF06}", cpu);
        assert!(lone, "braced surrogates stay lone");
        assert_eq!(out, [0xED, 0xA0, 0xB4, 0xED, 0xBC, 0x86]);
    }

    /// `\` + LS/PS is a LineContinuation: cooks to nothing (like `\<LF>`).
    #[test_cpus]
    fn cook_decode_ls_ps_line_continuation(cpu: impl Cpu) {
        let mut body = Vec::new();
        body.extend_from_slice(b"a\\");
        body.extend_from_slice("\u{2028}".as_bytes());
        body.extend_from_slice(b"b\\");
        body.extend_from_slice("\u{2029}".as_bytes());
        body.extend_from_slice(b"c");
        assert_eq!(cook(&body, cpu).0, b"abc");
        // a raw LS/PS is content, not a continuation - it must survive
        let mut raw = Vec::new();
        raw.extend_from_slice(b"a");
        raw.extend_from_slice("\u{2028}".as_bytes());
        raw.extend_from_slice(b"b");
        assert_eq!(cook(&raw, cpu).0, raw);
    }

    /// Template TV: raw <CR><LF> and <CR> normalize to <LF>; escape-produced
    /// `\r` survives; the string monomorph (CRLF=false) never normalizes.
    #[test_cpus]
    fn cook_decode_template_cr_normalization(cpu: impl Cpu) {
        /// Cook template `body` with [`cook_decode`] and CPU feature set `cpu`,
        /// and return the cooked bytes.
        #[cpu_multiversion]
        fn cook_tpl(body: &[u8], cpu: impl Cpu) -> Vec<u8> {
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
                    cpu,
                )
            };
            out.truncate(wi as usize);
            out
        }
        assert_eq!(cook_tpl(b"a\r\nb", cpu), b"a\nb");
        assert_eq!(cook_tpl(b"a\rb", cpu), b"a\nb");
        assert_eq!(cook_tpl(b"a\r\r\nb", cpu), b"a\n\nb");
        // Escape-produced CR is content, not a line ending.
        assert_eq!(cook_tpl(br"a\rb", cpu), b"a\rb");
        // `\<CR><LF>` line continuation still cooks to nothing.
        assert_eq!(cook_tpl(b"a\\\r\nb", cpu), b"ab");
        // The string monomorph is untouched by design.
        assert_eq!(cook(b"a\r\nb", cpu).0, b"a\r\nb");
    }
}
