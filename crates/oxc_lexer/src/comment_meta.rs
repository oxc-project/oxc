pub const CONTENT_NONE: u8 = 0;
pub const CONTENT_LEGAL: u8 = 1;
pub const CONTENT_JSDOC: u8 = 2;
pub const CONTENT_JSDOC_LEGAL: u8 = 3;
pub const CONTENT_PURE: u8 = 4;
pub const CONTENT_NO_SIDE_EFFECTS: u8 = 5;
pub const CONTENT_WEBPACK: u8 = 6;
pub const CONTENT_VITE: u8 = 7;
pub const CONTENT_COVERAGE_IGNORE: u8 = 8;
pub const META_MULTILINE: u8 = 0x10;

#[inline]
pub fn content_from_ordinal(o: u8) -> oxc_ast::ast::CommentContent {
    use oxc_ast::ast::CommentContent;
    match o & 0x0F {
        CONTENT_LEGAL => CommentContent::Legal,
        CONTENT_JSDOC => CommentContent::Jsdoc,
        CONTENT_JSDOC_LEGAL => CommentContent::JsdocLegal,
        CONTENT_PURE => CommentContent::Pure,
        CONTENT_NO_SIDE_EFFECTS => CommentContent::NoSideEffects,
        CONTENT_WEBPACK => CommentContent::Webpack,
        CONTENT_VITE => CommentContent::Vite,
        CONTENT_COVERAGE_IGNORE => CommentContent::CoverageIgnore,
        _ => CommentContent::None,
    }
}

#[inline]
fn body(src: &[u8], start: u32, end: u32, is_block: bool) -> &[u8] {
    let (lo, hi) = if is_block {
        (start as usize + 2, (end as usize).saturating_sub(2))
    } else {
        (start as usize + 2, end as usize)
    };
    let lo = lo.min(src.len());
    let hi = hi.clamp(lo, src.len());
    &src[lo..hi]
}

fn contains_license_or_preserve(hay: &[u8]) -> bool {
    if hay.len() < 9 {
        return false;
    }
    let lim = hay.len() - 8;
    let mut i = 0;
    while i < lim {
        if hay[i] == b'@' {
            match hay[i + 1] {
                b'l' if &hay[i + 2..i + 8] == b"icense" => return true,
                b'p' if &hay[i + 2..i + 9] == b"reserve" => return true,
                _ => {}
            }
        }
        i += 1;
    }
    false
}

fn has_nl_cr(bytes: &[u8]) -> bool {
    bytes.iter().any(|&b| b == b'\n' || b == b'\r')
}

#[inline]
fn classify(bytes: &[u8], is_block: bool, lic: bool) -> u8 {
    if bytes.is_empty() {
        return CONTENT_NONE;
    }
    match bytes[0] {
        b'!' => return CONTENT_LEGAL,
        b'*' if is_block => {
            if !bytes.iter().all(|&c| c == b'*') {
                return if lic { CONTENT_JSDOC_LEGAL } else { CONTENT_JSDOC };
            }
            return CONTENT_NONE;
        }
        _ => {}
    }

    let mut start = 0;
    while start < bytes.len() && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    if start >= bytes.len() {
        return CONTENT_NONE;
    }

    match bytes[start] {
        b'@' => {
            start += 1;
            if start >= bytes.len() {
                return CONTENT_NONE;
            }
            if bytes[start..].starts_with(b"vite") {
                return CONTENT_VITE;
            }
            if bytes[start..].starts_with(b"license") || bytes[start..].starts_with(b"preserve") {
                return CONTENT_LEGAL;
            }
        }
        b'#' => start += 1,
        b'w' => {
            if bytes[start..].starts_with(b"webpack")
                && start + 7 < bytes.len()
                && bytes[start + 7].is_ascii_uppercase()
            {
                return CONTENT_WEBPACK;
            }
            return if lic { CONTENT_LEGAL } else { CONTENT_NONE };
        }
        b'v' | b'c' | b'n' | b'i' => {
            let rest = &bytes[start..];
            if rest.starts_with(b"v8 ignore")
                || rest.starts_with(b"c8 ignore")
                || rest.starts_with(b"node:coverage")
                || rest.starts_with(b"istanbul ignore")
            {
                return CONTENT_COVERAGE_IGNORE;
            }
            return if lic { CONTENT_LEGAL } else { CONTENT_NONE };
        }
        _ => return if lic { CONTENT_LEGAL } else { CONTENT_NONE },
    }

    if start < bytes.len() && bytes[start..].starts_with(b"__") {
        let rest = &bytes[start + 2..];
        if rest.starts_with(b"PURE__") {
            return CONTENT_PURE;
        } else if rest.starts_with(b"NO_SIDE_EFFECTS__") {
            return CONTENT_NO_SIDE_EFFECTS;
        }
    }

    if lic { CONTENT_LEGAL } else { CONTENT_NONE }
}

#[inline]
pub fn meta_byte_flags(
    src: &[u8],
    start: u32,
    end: u32,
    is_block: bool,
    ml: bool,
    lic: bool,
) -> u8 {
    let ord = classify(body(src, start, end, is_block), is_block, lic) & 0x0F;
    ord | if ml && is_block { META_MULTILINE } else { 0 }
}

#[inline]
pub fn meta_byte_exact(src: &[u8], start: u32, end: u32, is_block: bool) -> u8 {
    let b = body(src, start, end, is_block);
    let lic = contains_license_or_preserve(b);
    let ord = classify(b, is_block, lic) & 0x0F;
    let ml = is_block && has_nl_cr(b);
    ord | if ml { META_MULTILINE } else { 0 }
}
