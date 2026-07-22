//! Port of typescript-go's `internal/vfs/vfsmatch/vfsmatch.go`.
//!
//! The tsconfig `include`/`exclude` file-matching algorithm: a component-wise glob matcher
//! (no regex, matching tsgo) plus a directory walk that prunes via a directory matcher.
//!
//! Specs arrive already absolute and normalized (from `oxc_resolver`), so — unlike tsgo — we
//! don't re-run `.`/`..` normalization. Matching is case-sensitive. Not yet ported: the
//! `.min.js` default exclusion and case-insensitive file systems.

use std::path::{Path, PathBuf};

use rustc_hash::FxHashSet;

use crate::tspath::{file_extension_is_one_of, get_directory_path, has_extension};

/// tsgo `IsImplicitGlob`: a directory-like last component (no `.`, `*`, `?`) implicitly
/// means `component/**/*`.
fn is_implicit_glob(last_component: &str) -> bool {
    !last_component.contains(['.', '*', '?'])
}

/// tsgo `isHiddenPath`.
fn is_hidden_path(name: &str) -> bool {
    name.starts_with('.')
}

/// tsgo `isPackageFolder`.
fn is_package_folder(name: &str) -> bool {
    name.eq_ignore_ascii_case("node_modules")
        || name.eq_ignore_ascii_case("jspm_packages")
        || name.eq_ignore_ascii_case("bower_components")
}

/// Absolute path → its segments, without the leading root or empty parts.
/// `/repo/src/a.ts` → `["repo", "src", "a.ts"]`.
fn path_segments(absolute: &str) -> Vec<&str> {
    absolute.split('/').filter(|segment| !segment.is_empty()).collect()
}

/// A piece of a wildcard component: `*.ts` → `[Star, Literal(".ts")]`.
enum Segment {
    Literal(String),
    Star,
    Question,
}

/// One component (path segment) of a compiled glob.
enum Component {
    Literal(String),
    Wildcard(Vec<Segment>),
    DoubleAsterisk,
}

/// A compiled glob pattern, matched component-by-component against path segments.
struct GlobPattern {
    components: Vec<Component>,
    is_exclude: bool,
}

impl GlobPattern {
    /// tsgo `compileGlobPattern`. Returns `None` for patterns that match nothing (an include
    /// ending in `**`).
    fn compile(spec: &str, is_exclude: bool) -> Option<Self> {
        let mut parts: Vec<String> =
            path_segments(spec).into_iter().map(ToString::to_string).collect();

        if !is_exclude && parts.last().map(String::as_str) == Some("**") {
            return None;
        }

        // A directory-like trailing component matches all files underneath it.
        if is_implicit_glob(parts.last().map_or("", String::as_str)) {
            parts.push("**".to_string());
            parts.push("*".to_string());
        }

        let components = parts.iter().map(|part| parse_component(part)).collect();
        Some(Self { components, is_exclude })
    }

    fn matches(&self, segments: &[&str], prefix_only: bool) -> bool {
        self.matches_at(segments, 0, 0, prefix_only)
    }

    /// tsgo `matchPathParts`.
    fn matches_at(
        &self,
        segments: &[&str],
        mut seg_idx: usize,
        mut comp_idx: usize,
        prefix_only: bool,
    ) -> bool {
        loop {
            if seg_idx >= segments.len() {
                return if prefix_only { true } else { self.pattern_satisfied(comp_idx) };
            }
            if comp_idx >= self.components.len() {
                return self.is_exclude && !prefix_only;
            }

            let part = segments[seg_idx];
            match &self.components[comp_idx] {
                Component::DoubleAsterisk => {
                    if self.matches_at(segments, seg_idx, comp_idx + 1, prefix_only) {
                        return true;
                    }
                    // `**` never descends into hidden or package folders for includes.
                    if !self.is_exclude && (is_hidden_path(part) || is_package_folder(part)) {
                        return false;
                    }
                    seg_idx += 1;
                    continue;
                }
                Component::Literal(literal) => {
                    if literal != part {
                        return false;
                    }
                }
                Component::Wildcard(wildcard_segments) => {
                    if !self.is_exclude && is_package_folder(part) {
                        return false;
                    }
                    if !self.match_wildcard(wildcard_segments, part) {
                        return false;
                    }
                }
            }
            seg_idx += 1;
            comp_idx += 1;
        }
    }

    /// tsgo `patternSatisfied`: remaining components can match empty input only if all `**`.
    fn pattern_satisfied(&self, comp_idx: usize) -> bool {
        self.components[comp_idx..].iter().all(|c| matches!(c, Component::DoubleAsterisk))
    }

    /// tsgo `matchWildcard`.
    fn match_wildcard(&self, segments: &[Segment], part: &str) -> bool {
        // Include-pattern wildcards at the start of a component never match hidden files.
        if !self.is_exclude
            && matches!(segments.first(), Some(Segment::Star | Segment::Question))
            && is_hidden_path(part)
        {
            return false;
        }
        match_segments(segments, part)
    }
}

/// tsgo `parseComponent`.
fn parse_component(part: &str) -> Component {
    if part == "**" {
        return Component::DoubleAsterisk;
    }
    if !part.contains(['*', '?']) {
        return Component::Literal(part.to_string());
    }
    Component::Wildcard(parse_segments(part))
}

/// tsgo `parseSegments`: `*.ts` → `[Star, Literal(".ts")]`.
fn parse_segments(part: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let bytes = part.as_bytes();
    let mut start = 0;
    for i in 0..bytes.len() {
        if bytes[i] == b'*' || bytes[i] == b'?' {
            if i > start {
                segments.push(Segment::Literal(part[start..i].to_string()));
            }
            segments.push(if bytes[i] == b'*' { Segment::Star } else { Segment::Question });
            start = i + 1;
        }
    }
    if start < bytes.len() {
        segments.push(Segment::Literal(part[start..].to_string()));
    }
    segments
}

/// tsgo `matchSegments`: iterative wildcard match with single-star backtracking, O(n*m).
fn match_segments(segments: &[Segment], part: &str) -> bool {
    let bytes = part.as_bytes();
    let (mut seg_idx, mut s_idx) = (0usize, 0usize);
    let (mut star_seg, mut star_s): (Option<usize>, usize) = (None, 0);

    while s_idx < bytes.len() {
        if let Some(segment) = segments.get(seg_idx) {
            match segment {
                Segment::Literal(literal) => {
                    let end = s_idx + literal.len();
                    if end <= bytes.len() && &bytes[s_idx..end] == literal.as_bytes() {
                        s_idx = end;
                        seg_idx += 1;
                        continue;
                    }
                }
                Segment::Question => {
                    if bytes[s_idx] != b'/' {
                        s_idx += char_width(bytes[s_idx]);
                        seg_idx += 1;
                        continue;
                    }
                }
                Segment::Star => {
                    star_seg = Some(seg_idx);
                    star_s = s_idx;
                    seg_idx += 1;
                    continue;
                }
            }
        }
        // Backtrack to the last star, letting it consume one more character.
        if let Some(star) = star_seg
            && star_s < bytes.len()
            && bytes[star_s] != b'/'
        {
            star_s += char_width(bytes[star_s]);
            s_idx = star_s;
            seg_idx = star + 1;
            continue;
        }
        return false;
    }

    while matches!(segments.get(seg_idx), Some(Segment::Star)) {
        seg_idx += 1;
    }
    seg_idx >= segments.len()
}

/// Byte length of the UTF-8 code point beginning with `first`.
fn char_width(first: u8) -> usize {
    match first {
        b if b < 0x80 => 1,
        b if b < 0xE0 => 2,
        b if b < 0xF0 => 3,
        _ => 4,
    }
}

/// tsgo `globMatcher`: include + exclude patterns.
struct GlobMatcher {
    includes: Vec<GlobPattern>,
    excludes: Vec<GlobPattern>,
    had_includes: bool,
}

impl GlobMatcher {
    fn new(include_specs: &[String], exclude_specs: &[String]) -> Self {
        Self {
            had_includes: !include_specs.is_empty(),
            includes: include_specs
                .iter()
                .filter_map(|spec| GlobPattern::compile(spec, false))
                .collect(),
            excludes: exclude_specs
                .iter()
                .filter_map(|spec| GlobPattern::compile(spec, true))
                .collect(),
        }
    }

    /// tsgo `matchesFileParts`: the index of the first include pattern the file matches (the
    /// result bucket it lands in), or `None` when excluded or unmatched.
    fn file_matches(&self, segments: &[&str]) -> Option<usize> {
        if self.excludes.iter().any(|pattern| pattern.matches(segments, false)) {
            return None;
        }
        if self.includes.is_empty() {
            return if self.had_includes { None } else { Some(0) };
        }
        self.includes.iter().position(|pattern| pattern.matches(segments, false))
    }

    /// tsgo `matchesDirectoryParts`: could any file under this directory match?
    fn directory_matches(&self, segments: &[&str]) -> bool {
        if self.excludes.iter().any(|pattern| pattern.matches(segments, false)) {
            return false;
        }
        if self.includes.is_empty() {
            return !self.had_includes;
        }
        self.includes.iter().any(|pattern| pattern.matches(segments, true))
    }
}

/// tsgo `getIncludeBasePath`: the non-wildcard base directory of an include spec.
fn get_include_base_path(absolute: &str) -> String {
    match absolute.find(['*', '?']) {
        None => {
            if has_extension(absolute) {
                get_directory_path(absolute).to_string()
            } else {
                absolute.to_string()
            }
        }
        Some(wildcard_offset) => {
            let cut = absolute[..wildcard_offset].rfind('/').unwrap_or(0);
            absolute[..cut].to_string()
        }
    }
}

/// tsgo `ContainsPath` (simplified): is `child` `parent` or nested under it?
fn contains_path(parent: &str, child: &str) -> bool {
    child == parent || child.strip_prefix(parent).is_some_and(|rest| rest.starts_with('/'))
}

/// tsgo `getBasePaths`: the base directories to start walking from.
fn get_base_paths(base: &Path, includes: &[String]) -> Vec<PathBuf> {
    let mut base_paths = vec![base.to_string_lossy().into_owned()];
    if !includes.is_empty() {
        let mut include_base_paths: Vec<String> =
            includes.iter().map(|include| get_include_base_path(include)).collect();
        include_base_paths.sort();
        for include_base_path in include_base_paths {
            if base_paths.iter().all(|existing| !contains_path(existing, &include_base_path)) {
                base_paths.push(include_base_path);
            }
        }
    }
    base_paths.into_iter().map(PathBuf::from).collect()
}

/// tsgo `ReadDirectory`/`matchFiles`: collect files under `base` matching the specs, filtered
/// by `extensions` (a flat suffix list). Directories are walked only when they could contain a
/// match, so `node_modules` and excluded subtrees are pruned.
///
/// Files are kept in one result bucket per include pattern and the buckets are concatenated in
/// include order (tsgo's `results [][]string`) — so a file's position follows the first include
/// spec it matches, not the raw walk order.
pub fn read_directory(
    base: &Path,
    extensions: &[&str],
    excludes: &[String],
    includes: &[String],
) -> Vec<PathBuf> {
    let matcher = GlobMatcher::new(includes, excludes);
    let mut visited_symlinks = FxHashSet::default();
    let mut results: Vec<Vec<PathBuf>> = vec![Vec::new(); matcher.includes.len().max(1)];
    for base_path in get_base_paths(base, includes) {
        visit(&base_path, &matcher, extensions, &mut visited_symlinks, &mut results);
    }
    results.into_iter().flatten().collect()
}

fn visit(
    dir: &Path,
    matcher: &GlobMatcher,
    extensions: &[&str],
    visited_symlinks: &mut FxHashSet<PathBuf>,
    results: &mut [Vec<PathBuf>],
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    let mut files: Vec<std::ffi::OsString> = Vec::new();
    let mut directories: Vec<(std::ffi::OsString, bool)> = Vec::new();
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let is_symlink = file_type.is_symlink();
        // Follow symlinks to classify the target; a broken link resolves to neither.
        let is_dir = if is_symlink {
            match std::fs::metadata(entry.path()) {
                Ok(metadata) => metadata.is_dir(),
                Err(_) => continue,
            }
        } else {
            file_type.is_dir()
        };
        // Everything that isn't a directory is a file candidate; the extension filter below
        // discards non-source entries.
        if is_dir {
            directories.push((entry.file_name(), is_symlink));
        } else {
            files.push(entry.file_name());
        }
    }
    files.sort();
    directories.sort();

    for name in &files {
        let Some(name) = name.to_str() else {
            continue;
        };
        if !extensions.is_empty() && !file_extension_is_one_of(name, extensions) {
            continue;
        }
        let full = dir.join(name);
        let full_string = full.to_string_lossy();
        if let Some(bucket) = matcher.file_matches(&path_segments(&full_string)) {
            drop(full_string);
            results[bucket].push(full);
        }
    }

    for (name, is_symlink) in &directories {
        let full = dir.join(name);
        let full_string = full.to_string_lossy();
        let should_recurse = matcher.directory_matches(&path_segments(&full_string));
        drop(full_string);
        if !should_recurse {
            continue;
        }
        if *is_symlink {
            // Guard against symlink cycles by tracking resolved targets.
            let canonical = std::fs::canonicalize(&full).unwrap_or_else(|_| full.clone());
            if !visited_symlinks.insert(canonical) {
                continue;
            }
        }
        visit(&full, matcher, extensions, visited_symlinks, results);
    }
}

/// tsgo `SpecMatcher`: does a path match any of the given include specs?
pub struct SpecMatcher {
    patterns: Vec<GlobPattern>,
}

impl SpecMatcher {
    pub fn new(specs: &[String]) -> Self {
        Self {
            patterns: specs.iter().filter_map(|spec| GlobPattern::compile(spec, false)).collect(),
        }
    }

    pub fn matches(&self, path: &Path) -> bool {
        let path = path.to_string_lossy();
        let segments = path_segments(&path);
        self.patterns.iter().any(|pattern| pattern.matches(&segments, false))
    }
}
