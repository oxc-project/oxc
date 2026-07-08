use core::arch::x86_64::*;

use crate::tables::{hex_val, is_digit, is_word};

#[inline(always)]
pub(super) unsafe fn veq(v: __m256i, c: u8) -> __m256i {
    _mm256_cmpeq_epi8(v, _mm256_set1_epi8(c as i8))
}
#[inline(always)]
pub(super) unsafe fn mm(v: __m256i) -> u32 {
    _mm256_movemask_epi8(v) as u32
}
#[inline(always)]
pub(super) unsafe fn load256(src: *const u8, i: usize) -> __m256i {
    _mm256_loadu_si256(src.add(i) as *const __m256i)
}
#[inline]
pub(super) unsafe fn find1(src: *const u8, n: usize, mut i: usize, a: u8) -> usize {
    while i + 32 <= n {
        let m = mm(veq(load256(src, i), a));
        if m != 0 {
            return i + m.trailing_zeros() as usize;
        }
        i += 32;
    }
    while i < n {
        if *src.add(i) == a {
            return i;
        }
        i += 1;
    }
    n
}
#[inline]
pub(super) unsafe fn find2(src: *const u8, n: usize, mut i: usize, a: u8, b: u8) -> usize {
    while i + 32 <= n {
        let v = load256(src, i);
        let m = mm(_mm256_or_si256(veq(v, a), veq(v, b)));
        if m != 0 {
            return i + m.trailing_zeros() as usize;
        }
        i += 32;
    }
    while i < n {
        let c = *src.add(i);
        if c == a || c == b {
            return i;
        }
        i += 1;
    }
    n
}
#[inline]
unsafe fn find3(src: *const u8, n: usize, mut i: usize, a: u8, b: u8, c: u8) -> usize {
    while i + 32 <= n {
        let v = load256(src, i);
        let m = mm(_mm256_or_si256(_mm256_or_si256(veq(v, a), veq(v, b)), veq(v, c)));
        if m != 0 {
            return i + m.trailing_zeros() as usize;
        }
        i += 32;
    }
    while i < n {
        let ch = *src.add(i);
        if ch == a || ch == b || ch == c {
            return i;
        }
        i += 1;
    }
    n
}
#[inline]
unsafe fn find4(src: *const u8, n: usize, mut i: usize, a: u8, b: u8, c: u8, d: u8) -> usize {
    while i + 32 <= n {
        let v = load256(src, i);
        let m = mm(_mm256_or_si256(
            _mm256_or_si256(veq(v, a), veq(v, b)),
            _mm256_or_si256(veq(v, c), veq(v, d)),
        ));
        if m != 0 {
            return i + m.trailing_zeros() as usize;
        }
        i += 32;
    }
    while i < n {
        let x = *src.add(i);
        if x == a || x == b || x == c || x == d {
            return i;
        }
        i += 1;
    }
    n
}
/// First ECMAScript LineTerminator at/after `i`: LF, CR, or the 3-byte LS/PS
/// (U+2028/U+2029). Stops at LS/PS too, so the hashbang scan cannot run
/// through one. A 0xE2 that isn't LS/PS keeps scanning; the 2-byte confirm
/// past a trailing 0xE2 reads the pad, which can never match 0x80.
#[inline]
pub(super) unsafe fn find_line_terminator(src: *const u8, n: usize, mut i: usize) -> usize {
    loop {
        let p = find3(src, n, i, b'\n', b'\r', 0xE2);
        if p >= n {
            return n;
        }
        if *src.add(p) != 0xE2 {
            return p;
        }
        if *src.add(p + 1) == 0x80 && (*src.add(p + 2) == 0xA8 || *src.add(p + 2) == 0xA9) {
            return p;
        }
        i = p + 1;
    }
}
/// OR-fold of `vpcmpeqb` results, associated as a tree.
macro_rules! vor {
    ($a:expr) => { $a };
    ($a:expr, $b:expr) => { _mm256_or_si256($a, $b) };
    ($a:expr, $b:expr, $($rest:expr),+) => {
        _mm256_or_si256(vor!($a, $b), vor!($($rest),+))
    };
}

/// Define a fixed-needle SIMD finder: `$name(src, n, i)` returns the index
/// of the first byte in `src[i..n]` matching any needle, or `n`.
macro_rules! simd_finder {
    ($(#[$attr:meta])* $name:ident: $($needle:expr),+ $(,)?) => {
        $(#[$attr])*
        #[inline]
        pub(super) unsafe fn $name(src: *const u8, n: usize, mut i: usize) -> usize {
            while i + 32 <= n {
                let v = load256(src, i);
                let m = mm(vor!($(veq(v, $needle)),+));
                if m != 0 {
                    return i + m.trailing_zeros() as usize;
                }
                i += 32;
            }
            while i < n {
                let c = *src.add(i);
                if $(c == $needle)||+ {
                    return i;
                }
                i += 1;
            }
            n
        }
    };
}

simd_finder!(
    /// JS-mode top-level scan: string/template/regex-or-comment openers plus
    /// the Annex B `<!--` / `-->` trigger bytes.
    find_opener: b'"', b'\'', b'`', b'/', b'<', b'>'
);
simd_finder!(
    /// [`find_opener`] widened with `{` / `}` — used inside template
    /// substitutions, where braces drive the nesting depth.
    find_opener6: b'"', b'\'', b'`', b'/', b'{', b'}', b'<', b'>'
);
simd_finder!(
    /// [`find_opener`] shape for `carve_jsx` JS mode at top level: `<` (the
    /// JSX-start byte) instead of the Annex B `<` / `>` pair.
    find_opener_jsx5: b'"', b'\'', b'`', b'/', b'<'
);
simd_finder!(
    /// [`find_opener_jsx5`] widened with `{` / `}`. Used by `carve_jsx` JS
    /// mode inside a template substitution or JSX expression container.
    find_opener_jsx7: b'"', b'\'', b'`', b'/', b'{', b'}', b'<'
);
simd_finder!(
    /// TAG-mode scan: the bytes that matter inside an opening `<...>` tag.
    find_jsx_tag: b'"', b'\'', b'{', b'/', b'>'
);
simd_finder!(
    /// [`find_jsx_tag`] widened with `<` (`.tsx` only), so `carve_jsx` can
    /// spot and skip a type-argument list inside the opening tag.
    find_jsx_tag_ts: b'"', b'\'', b'{', b'/', b'>', b'<'
);
simd_finder!(
    /// TEXT-mode scan (strict): JSX child text ends at any of `< { > }`.
    find_jsx_text: b'<', b'{', b'>', b'}'
);
simd_finder!(
    /// Template-body scan: terminator, escape lead, or `$` (`${` starts a
    /// substitution).
    find_tmpl: b'`', b'\\', b'$'
);
simd_finder!(
    /// Regex-body scan. LF/CR and the 0xE2 lead (LS/PS) are watched so
    /// line terminators in the body can be diagnosed.
    find_regex: b'/', b'\\', b'[', b']', b'\n', b'\r', 0xE2
);
#[inline]
pub(super) unsafe fn scan_quoted(
    src: *const u8,
    n: usize,
    mut i: usize,
    q: u8,
    saw_nl: &mut bool,
) -> usize {
    loop {
        // LF/CR are watched too: raw line terminators in strings are diagnosed.
        let p = find4(src, n, i, q, b'\\', b'\n', b'\r');
        if p >= n {
            return n;
        }
        let ch = *src.add(p);
        if ch == b'\\' {
            // Escape, including line continuations. `\<CR><LF>` is one
            // LineTerminatorSequence: skip all three bytes, or the next find
            // lands on the LF and flags a legal continuation.
            let crlf = *src.add(p + 1) == b'\r' && *src.add(p + 2) == b'\n';
            i = p + 2 + usize::from(crlf);
            continue;
        }
        if ch == b'\n' || ch == b'\r' {
            *saw_nl = true;
            i = p + 1; // keep scanning so the token end is unchanged
            continue;
        }
        return p;
    }
}
#[inline(always)]
unsafe fn lic_verify_at(src: *const u8, q: usize) -> bool {
    let c1 = *src.add(q + 1);
    // spellchecker:disable-next-line
    (c1 == b'l' && core::slice::from_raw_parts(src.add(q + 2), 6) == b"icense")
        || (c1 == b'p' && core::slice::from_raw_parts(src.add(q + 2), 7) == b"reserve")
}
#[inline(always)]
unsafe fn lic_first(src: *const u8, base: usize, mut am: u32) -> i64 {
    while am != 0 {
        let q = base + am.trailing_zeros() as usize;
        am &= am - 1;
        if lic_verify_at(src, q) {
            return q as i64;
        }
    }
    -1
}
pub(super) unsafe fn scan_block_comment(
    src: *const u8,
    n: usize,
    mut i: usize,
) -> (usize, bool, i64) {
    let mut saw_nl = false;
    let mut lic_q: i64 = -1;
    while i + 32 <= n {
        let v = load256(src, i);
        let vn = load256(src, i + 1);
        let term = mm(_mm256_and_si256(veq(v, b'*'), veq(vn, b'/')));
        let nl = mm(_mm256_or_si256(veq(v, b'\n'), veq(v, b'\r')));
        let at = mm(veq(v, b'@'));
        if term != 0 {
            let j = term.trailing_zeros() as usize;
            let bodymask: u32 = if j > 0 { (1u32 << j) - 1 } else { 0 };
            saw_nl |= (nl & bodymask) != 0;
            if lic_q < 0 {
                let am = at & bodymask;
                if am != 0 {
                    lic_q = lic_first(src, i, am);
                }
            }
            return (i + j + 1, saw_nl, lic_q);
        }
        saw_nl |= nl != 0;
        if lic_q < 0 && at != 0 {
            lic_q = lic_first(src, i, at);
        }
        i += 32;
    }
    while i + 1 < n {
        let c = *src.add(i);
        if c == b'*' && *src.add(i + 1) == b'/' {
            return (i + 1, saw_nl, lic_q);
        }
        if c == b'\n' || c == b'\r' {
            saw_nl = true;
        }
        if c == b'@' && lic_q < 0 && lic_verify_at(src, i) {
            lic_q = i as i64;
        }
        i += 1;
    }
    (n, saw_nl, lic_q)
}
/// Scan a `//` line comment to its LineTerminator (LF, CR, or LS/PS),
/// tracking the first `@license` / `@preserve` position for comment
/// metadata. A 0xE2 that isn't LS/PS (typographic punctuation in prose) is
/// cleared from the mask and the scan continues; the LS/PS confirm can read
/// the pad, which never matches 0x80.
pub(super) unsafe fn scan_line_comment(src: *const u8, n: usize, mut i: usize) -> (usize, i64) {
    let mut lic_q: i64 = -1;
    while i + 32 <= n {
        let v = load256(src, i);
        let term_v = _mm256_or_si256(_mm256_or_si256(veq(v, b'\n'), veq(v, b'\r')), veq(v, 0xE2));
        let mut term = mm(term_v);
        let at = mm(veq(v, b'@'));
        while term != 0 {
            let t = term.trailing_zeros() as usize;
            let c = *src.add(i + t);
            if c == 0xE2
                && !(*src.add(i + t + 1) == 0x80
                    && (*src.add(i + t + 2) == 0xA8 || *src.add(i + t + 2) == 0xA9))
            {
                term &= term - 1; // not LS/PS: clear and keep scanning
                continue;
            }
            if lic_q < 0 {
                let am = at & if t > 0 { (1u32 << t) - 1 } else { 0 };
                if am != 0 {
                    lic_q = lic_first(src, i, am);
                }
            }
            return (i + t, lic_q);
        }
        if lic_q < 0 && at != 0 {
            lic_q = lic_first(src, i, at);
        }
        i += 32;
    }
    while i < n {
        let c = *src.add(i);
        if c == b'\n' || c == b'\r' {
            return (i, lic_q);
        }
        if c == 0xE2
            && *src.add(i + 1) == 0x80
            && (*src.add(i + 2) == 0xA8 || *src.add(i + 2) == 0xA9)
        {
            return (i, lic_q);
        }
        if c == b'@' && lic_q < 0 && lic_verify_at(src, i) {
            lic_q = i as i64;
        }
        i += 1;
    }
    (n, lic_q)
}
#[inline]
pub(super) unsafe fn scan_regex(
    src: *const u8,
    n: usize,
    mut i: usize,
    nl_at: &mut usize,
) -> usize {
    let mut incl = false;
    loop {
        let p = find_regex(src, n, i);
        if p >= n {
            return n;
        }
        let c = *src.add(p);
        if c == b'\\' {
            // An escaped line terminator is still invalid in a regex (the
            // grammar requires a NonTerminator); record it here or the p+2
            // skip would sail past it. The p+1<n guard keeps garbage pad
            // bytes after a `\` at EOF out of the diagnostics.
            if p + 1 < n {
                let c1 = *src.add(p + 1);
                if (c1 == b'\n' || c1 == b'\r') && *nl_at == usize::MAX {
                    *nl_at = p + 1;
                } else if c1 == 0xE2
                    && *nl_at == usize::MAX
                    && p + 3 < n
                    && *src.add(p + 2) == 0x80
                    && (*src.add(p + 3) == 0xA8 || *src.add(p + 3) == 0xA9)
                {
                    // Escaped LS/PS: span ends past the whole char; the p+2
                    // skip lands mid-sequence (unwatched 0x80) and sails on.
                    *nl_at = p + 3;
                }
            }
            i = p + 2;
            continue;
        }
        if c == b'\n' || c == b'\r' {
            // Raw line terminator: invalid anywhere in the body, `[...]`
            // included. Record the first; keep scanning so the token end
            // (and every downstream span) is unchanged.
            if *nl_at == usize::MAX {
                *nl_at = p;
            }
            i = p + 1;
            continue;
        }
        if c == 0xE2 {
            // LS/PS are line terminators too; other E2-led chars (em dash,
            // arrows) fall through untouched.
            if *nl_at == usize::MAX
                && p + 2 < n
                && *src.add(p + 1) == 0x80
                && (*src.add(p + 2) == 0xA8 || *src.add(p + 2) == 0xA9)
            {
                *nl_at = p + 2; // span ends past the whole char
            }
            i = p + 1;
            continue;
        }
        if incl {
            if c == b']' {
                incl = false;
            }
            i = p + 1;
            continue;
        }
        if c == b'[' {
            incl = true;
            i = p + 1;
            continue;
        }
        if c == b'/' {
            return p;
        }
        i = p + 1;
    }
}
#[inline]
pub(super) unsafe fn scan_tmpl_text(
    src: *const u8,
    n: usize,
    mut i: usize,
    term: &mut i32,
) -> usize {
    loop {
        let p = find_tmpl(src, n, i);
        if p >= n {
            *term = 0;
            return n;
        }
        let c = *src.add(p);
        if c == b'\\' {
            i = p + 2;
            continue;
        }
        if c == b'`' {
            *term = 1;
            return p + 1;
        }
        if *src.add(p + 1) == b'{' {
            *term = 2;
            return p + 2;
        }
        i = p + 1;
    }
}
#[inline]
pub(super) unsafe fn scan_number(src: *const u8, n: usize, pos: usize) -> usize {
    if *src.add(pos) == b'0' && pos + 1 < n {
        let c = *src.add(pos + 1) | 0x20;
        if c == b'x' || c == b'o' || c == b'b' {
            let radix = if c == b'x' {
                16
            } else if c == b'o' {
                8
            } else {
                2
            };
            let mut i = pos + 2;
            while i < n {
                let d = *src.add(i);
                if d == b'_' {
                    i += 1;
                    continue;
                }
                if hex_val(d) < radix {
                    i += 1;
                } else {
                    break;
                }
            }
            if i < n && *src.add(i) == b'n' {
                return i + 1;
            }
            return i;
        }
    }
    let mut i = pos;
    let mut is_float = false;
    while i < n && (is_digit(*src.add(i)) || *src.add(i) == b'_') {
        i += 1;
    }
    if i < n && *src.add(i) == b'.' {
        is_float = true;
        i += 1;
        while i < n && (is_digit(*src.add(i)) || *src.add(i) == b'_') {
            i += 1;
        }
    }
    if i < n && (*src.add(i) | 0x20) == b'e' {
        is_float = true;
        i += 1;
        if i < n && (*src.add(i) == b'+' || *src.add(i) == b'-') {
            i += 1;
        }
        while i < n && (is_digit(*src.add(i)) || *src.add(i) == b'_') {
            i += 1;
        }
    }
    if !is_float && i < n && *src.add(i) == b'n' {
        return i + 1;
    }
    i
}
#[inline]
pub(super) unsafe fn scan_ident_esc(src: *const u8, n: usize, p: usize) -> usize {
    let mut i = p;
    while i < n {
        if *src.add(i) == b'\\' && i + 1 < n && *src.add(i + 1) == b'u' {
            i += 2;
            if i < n && *src.add(i) == b'{' {
                i += 1;
                while i < n && hex_val(*src.add(i)) != 255 {
                    i += 1;
                }
                if i < n && *src.add(i) == b'}' {
                    i += 1;
                }
            } else {
                let mut k = 0;
                while k < 4 && i < n && hex_val(*src.add(i)) != 255 {
                    i += 1;
                    k += 1;
                }
            }
            while i < n && is_word(*src.add(i)) {
                i += 1;
            }
            continue;
        }
        break;
    }
    i
}
