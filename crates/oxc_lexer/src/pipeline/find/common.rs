use super::{find3, finder};

finder!(
    /// JS-mode top-level scan: string/template/regex-or-comment openers plus
    /// the Annex B `<!--` / `-->` trigger bytes.
    find_opener: b'"', b'\'', b'`', b'/', b'<', b'>'
);

finder!(
    /// [`find_opener`] widened with `{` / `}` — used inside template
    /// substitutions, where braces drive the nesting depth.
    find_opener6: b'"', b'\'', b'`', b'/', b'{', b'}', b'<', b'>'
);

finder!(
    /// [`find_opener`] shape for `carve_jsx` JS mode at top level: `<` (the
    /// JSX-start byte) instead of the Annex B `<` / `>` pair.
    find_opener_jsx5: b'"', b'\'', b'`', b'/', b'<'
);

finder!(
    /// [`find_opener_jsx5`] widened with `{` / `}`. Used by `carve_jsx` JS
    /// mode inside a template substitution or JSX expression container.
    find_opener_jsx7: b'"', b'\'', b'`', b'/', b'{', b'}', b'<'
);

finder!(
    /// TAG-mode scan: the bytes that matter inside an opening `<...>` tag.
    /// Deliberately not widened with `-` for hyphenated JSXIdentifiers: an
    /// extra needle costs a broadcast in every call, and TAG mode calls this
    /// once per attribute.
    find_jsx_tag: b'"', b'\'', b'{', b'/', b'>', b'<'
);

finder!(
    /// TEXT-mode scan (strict): JSX child text ends at any of `< { > }`.
    find_jsx_text: b'<', b'{', b'>', b'}'
);

finder!(
    /// Template-body scan: terminator, escape lead, or `$` (`${` starts a
    /// substitution).
    find_tmpl: b'`', b'\\', b'$'
);

finder!(
    /// Regex-body scan. LF/CR and the 0xE2 lead (LS/PS) are watched so
    /// line terminators in the body can be diagnosed.
    find_regex: b'/', b'\\', b'[', b']', b'\n', b'\r', 0xE2
);

/// First ECMAScript LineTerminator at/after `i`: LF, CR, or the 3-byte LS/PS
/// (U+2028/U+2029). Stops at LS/PS too, so the hashbang scan cannot run
/// through one. A 0xE2 that isn't LS/PS keeps scanning; the 2-byte confirm
/// past a trailing 0xE2 reads the pad, which can never match 0x80.
#[inline]
pub unsafe fn find_line_terminator(src: *const u8, n: usize, mut i: usize) -> usize {
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

/// Byte length (2 or 3) of the multi-byte ECMAScript WhiteSpace /
/// LineTerminator at `p`, or 0. The non-ASCII set: U+0085, U+00A0, U+1680,
/// U+2000..=U+200B, U+2028, U+2029, U+202F, U+205F, U+3000, U+FEFF.
#[inline]
pub unsafe fn unicode_ws_len(src: *const u8, p: usize) -> usize {
    let c1 = *src.add(p + 1);
    match *src.add(p) {
        0xC2 => usize::from(c1 == 0xA0 || c1 == 0x85) * 2,
        0xE1 => usize::from(c1 == 0x9A && *src.add(p + 2) == 0x80) * 3,
        0xE2 => {
            let c2 = *src.add(p + 2);
            let is_ws = (c1 == 0x80
                && ((0x80..=0x8B).contains(&c2) || c2 == 0xA8 || c2 == 0xA9 || c2 == 0xAF))
                || (c1 == 0x81 && c2 == 0x9F);
            usize::from(is_ws) * 3
        }
        0xE3 => usize::from(c1 == 0x80 && *src.add(p + 2) == 0x80) * 3,
        0xEF => usize::from(c1 == 0xBB && *src.add(p + 2) == 0xBF) * 3,
        _ => 0,
    }
}
