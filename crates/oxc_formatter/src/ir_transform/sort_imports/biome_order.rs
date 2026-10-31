//! Import order compatible with Biome's `organizeImports` assist.
//!
//! Ported from Biome (MIT License, Copyright (c) 2023-present Biome Developers and Contributors):
//! - `biome_rule_options::organize_imports::import_source`
//! - `biome_string_case::Collator`
//! - `biome_js_analyze::assist::source::organize_imports::import_key`
//!
//! Sources are ordered by kind first (URL, protocol package, package, alias, path),
//! then by a natural, case-interleaved collation (`A < a < B < b`) where `/` sorts lowest.
//! Paths are compared component-wise so that farther paths come first (`../..` < `..` < `.`).

use std::cmp::Ordering;

use super::source_line::ImportLineMetadata;

/// Compare two import sources the way Biome does.
pub fn compare_sources(a: &str, b: &str) -> Ordering {
    let kind_a = SourceKind::from_source(a);
    let kind_b = SourceKind::from_source(b);
    kind_a.cmp(&kind_b).then_with(|| {
        if kind_a == SourceKind::Path {
            // Components are normalized, e.g. `./..` and `..` are equal.
            // Fall back to the string order to keep the order total.
            PathComponents::new(a)
                .cmp(PathComponents::new(b))
                .then_with(|| collate(a.as_bytes(), b.as_bytes()))
        } else {
            collate(a.as_bytes(), b.as_bytes())
        }
    })
}

/// Rank used to order imports that share the same source, matching Biome:
/// type imports first, then namespace, default and named imports.
pub fn statement_kind_rank(metadata: &ImportLineMetadata) -> u8 {
    let default = metadata.has_default_specifier;
    let namespace = metadata.has_namespace_specifier;
    if metadata.is_type_import {
        return if namespace {
            0
        } else if default {
            1
        } else {
            2
        };
    }
    match (default, namespace, metadata.has_named_specifier) {
        (false, true, _) => 3,
        (true, true, _) => 4,
        (true, false, false) => 5,
        (true, false, true) => 6,
        _ => 7,
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum SourceKind {
    Unknown,
    /// `https://example.org`
    Url,
    /// `node:test`, `npm:@scope/package`
    ProtocolPackage,
    /// `package`, `@scope/package`
    Package,
    /// Sources that start with `@/`, `#`, `~`, `$`, or `%`
    Alias,
    /// Sources that start with `/`, `./`, or `../`
    Path,
}

impl SourceKind {
    fn from_source(source: &str) -> Self {
        let mut iter = source.bytes();
        match iter.next() {
            Some(b'@') => match iter.next() {
                Some(b'/') | None => Self::Alias,
                Some(b'a'..=b'z' | b'0'..=b'9' | b'-') => Self::Package,
                _ => Self::Unknown,
            },
            Some(b'#' | b'~' | b'$' | b'%') => Self::Alias,
            Some(b'/') => Self::Path,
            Some(b'.') => match iter.next() {
                Some(b'.') => {
                    if matches!(iter.next(), None | Some(b'/')) {
                        Self::Path
                    } else {
                        Self::Unknown
                    }
                }
                None | Some(b'/') => Self::Path,
                Some(_) => Self::Unknown,
            },
            Some(b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_') => loop {
                match iter.next() {
                    Some(b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.') => {}
                    Some(b':') => {
                        return match iter.next() {
                            Some(b'/') => Self::Url,
                            Some(b'@' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_') => {
                                Self::ProtocolPackage
                            }
                            _ => Self::Unknown,
                        };
                    }
                    None | Some(b'/') => return Self::Package,
                    Some(_) => return Self::Unknown,
                }
            },
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PathComponent<'a> {
    RootDir,
    /// One or more consecutive `..`
    ParentDir(usize),
    CurDir,
    Normal(&'a [u8]),
}

impl PartialOrd for PathComponent<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathComponent<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        fn rank(component: &PathComponent) -> u8 {
            match component {
                PathComponent::RootDir => 0,
                PathComponent::ParentDir(_) => 1,
                PathComponent::CurDir => 2,
                PathComponent::Normal(_) => 3,
            }
        }
        match (self, other) {
            // Farther parents come first.
            (Self::ParentDir(n1), Self::ParentDir(n2)) => n2.cmp(n1),
            (Self::Normal(s1), Self::Normal(s2)) => collate(s1, s2),
            _ => rank(self).cmp(&rank(other)),
        }
    }
}

/// Splits an import path into components like `std::path::Components` does on Unix,
/// independently of the host platform, and merges consecutive `..` into one component.
struct PathComponents<'a> {
    rest: Option<&'a [u8]>,
    at_start: bool,
}

impl<'a> PathComponents<'a> {
    fn new(path: &'a str) -> Self {
        Self { rest: Some(path.as_bytes()), at_start: true }
    }

    fn peek_segment(&self) -> Option<&'a [u8]> {
        let rest = self.rest?;
        Some(rest.iter().position(|b| *b == b'/').map_or(rest, |end| &rest[..end]))
    }

    fn next_segment(&mut self) -> Option<&'a [u8]> {
        let rest = self.rest?;
        if let Some(end) = rest.iter().position(|b| *b == b'/') {
            self.rest = Some(&rest[end + 1..]);
            Some(&rest[..end])
        } else {
            self.rest = None;
            Some(rest)
        }
    }

    fn count_parent_dirs(&mut self) -> usize {
        let mut count = 0;
        while let Some(segment) = self.peek_segment() {
            match segment {
                b".." => count += 1,
                b"" | b"." => {}
                _ => break,
            }
            self.next_segment();
        }
        count
    }
}

impl<'a> Iterator for PathComponents<'a> {
    type Item = PathComponent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let at_start = std::mem::replace(&mut self.at_start, false);
        loop {
            let segment = self.next_segment()?;
            match segment {
                b"" if at_start => return Some(PathComponent::RootDir),
                b"." if at_start => {
                    // Normalize `./../..` to `../..`.
                    let parent_dirs = self.count_parent_dirs();
                    return Some(if parent_dirs == 0 {
                        PathComponent::CurDir
                    } else {
                        PathComponent::ParentDir(parent_dirs)
                    });
                }
                b"" | b"." => {}
                b".." => return Some(PathComponent::ParentDir(1 + self.count_parent_dirs())),
                _ => return Some(PathComponent::Normal(segment)),
            }
        }
    }
}

/// Collation weights: control characters, then `/` and other punctuation,
/// then digits, then letters with uppercase directly before lowercase.
const COLLATION: [u8; 128] = [
    b'\0', 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13,
    0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x7f, b'\t', b'\n',
    0x0b, 0x0c, b'\r', b'/', b'\\', b'?', b'#', b'=', b'&', b';', b',', b'@', b':', b'.', b' ',
    b'_', b'-', b'+', b'*', b'!', b'%', b'$', b'(', b')', b'[', b']', b'{', b'}', b'<', b'>', b'|',
    b'^', b'~', b'\'', b'"', b'`', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
    b'A', b'a', b'B', b'b', b'C', b'c', b'D', b'd', b'E', b'e', b'F', b'f', b'G', b'g', b'H', b'h',
    b'I', b'i', b'J', b'j', b'K', b'k', b'L', b'l', b'M', b'm', b'N', b'n', b'O', b'o', b'P', b'p',
    b'Q', b'q', b'R', b'r', b'S', b's', b'T', b't', b'U', b'u', b'V', b'v', b'W', b'w', b'X', b'x',
    b'Y', b'y', b'Z', b'z',
];

const WEIGHTS: [u8; 256] = {
    let mut weights = [0u8; 256];
    let mut weight: u8 = 0;
    while weight < 128 {
        weights[COLLATION[weight as usize] as usize] = weight;
        weight += 1;
    }
    loop {
        weights[weight as usize] = weight;
        if weight == u8::MAX {
            break;
        }
        weight += 1;
    }
    weights
};

/// Natural comparison with [`COLLATION`] weights: digit runs compare by numeric value (`9` < `10`).
fn collate(s1: &[u8], s2: &[u8]) -> Ordering {
    let mut iter1 = s1.iter().copied();
    let mut iter2 = s2.iter().copied();
    let mut prev: Option<u8> = None;
    loop {
        match (iter1.next(), iter2.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(c1), Some(c2)) if c1 == c2 => prev = Some(c1),
            (Some(mut c1), Some(mut c2)) => {
                let prev_is_digit = |prev: Option<u8>| prev.is_some_and(|c| c.is_ascii_digit());
                let mut number_ordering = Ordering::Equal;
                loop {
                    match (c1.is_ascii_digit(), c2.is_ascii_digit()) {
                        (false, false) => {
                            if number_ordering == Ordering::Equal {
                                break;
                            }
                            return number_ordering;
                        }
                        (false, true) => {
                            if prev_is_digit(prev) {
                                return Ordering::Less;
                            }
                            break;
                        }
                        (true, false) => {
                            if prev_is_digit(prev) {
                                return Ordering::Greater;
                            }
                            break;
                        }
                        (true, true) => number_ordering = number_ordering.then(c1.cmp(&c2)),
                    }
                    match (iter1.next(), iter2.next()) {
                        (None, None) => return number_ordering,
                        (None, Some(next2)) => {
                            return if next2.is_ascii_digit() {
                                Ordering::Less
                            } else {
                                number_ordering.then(Ordering::Less)
                            };
                        }
                        (Some(next1), None) => {
                            return if next1.is_ascii_digit() {
                                Ordering::Greater
                            } else {
                                number_ordering.then(Ordering::Greater)
                            };
                        }
                        (Some(next1), Some(next2)) => {
                            prev = Some(c1);
                            c1 = next1;
                            c2 = next2;
                        }
                    }
                }
                let ordering = WEIGHTS[c1 as usize].cmp(&WEIGHTS[c2 as usize]);
                if ordering != Ordering::Equal {
                    return ordering;
                }
                prev = Some(c1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_sorted(sorted: &[&str]) {
        for pair in sorted.windows(2) {
            assert_eq!(compare_sources(pair[0], pair[1]), Ordering::Less, "{pair:?}");
            assert_eq!(compare_sources(pair[1], pair[0]), Ordering::Greater, "{pair:?}");
        }
    }

    #[test]
    fn orders_by_source_kind() {
        assert_sorted(&[
            ":alias",
            "https://example.org",
            "bun:test",
            "node:test",
            "@scope/package",
            "package",
            "#internal",
            "@/internal",
            "~/internal",
            "/",
            "../..",
            "..",
            ".",
        ]);
    }

    #[test]
    fn orders_paths_by_proximity() {
        assert_sorted(&[
            "../../../helpers/render",
            "../..",
            "..",
            "../a",
            ".",
            "./a",
            "./a/b",
            "./a-b",
            "./b",
        ]);
        assert_sorted(&[".././..", "../..", "./..", ".."]);
    }

    #[test]
    fn collates_naturally() {
        assert_sorted(&[":hyperloop/core", ":hyperloop-airbnb/core"]);
        assert_sorted(&[
            "./PopOver",
            "./parts",
            "./path9",
            "./path10",
            "./Tab",
            "./THead",
            "./Th",
            "./tab",
        ]);
        for pair in ["a.js", "a a.js", "a_a.js", "a-a.js", "a+a.js", "a*a.js"].windows(2) {
            assert_eq!(collate(pair[0].as_bytes(), pair[1].as_bytes()), Ordering::Less, "{pair:?}");
        }
        assert_eq!(compare_sources("./mock.json", "./mock-config.json"), Ordering::Less);
    }
}
