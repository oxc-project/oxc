//! Ported from typescript-go `internal/vfs/vfsmatch/vfsmatch.go`.
//!
//! A regex-free glob matcher + directory walker. This is the heart of "get all the
//! TypeScript files for a project": given `include`/`exclude` specs and a set of supported
//! extensions, it walks the filesystem and returns the matching files, encoding the
//! package-folder / `.min.js` / dotfile exclusions structurally (tsgo moved off the old
//! regex approach, and Rust's `regex` cannot express its negative look-aheads anyway).
//!
//! The [`FileSystemHost`] trait abstracts the filesystem so the matcher can be unit-tested
//! against an in-memory tree; [`StdFs`] is the real `std::fs` implementation.

use cow_utils::CowUtils;
use rustc_hash::FxHashSet;

use crate::tspath::TsPath;

/// How a set of patterns is being used, which changes matching rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Usage {
    /// Matching concrete files (applies the `.min.js` default exclusion).
    Files,
    /// Matching directories to decide whether to descend into them.
    Directories,
    /// Matching exclude patterns (looser trailing-`**` rules, no package-folder skipping).
    Exclude,
}

/// Pass as the `depth` argument to [`read_directory`] for no depth limit.
pub const UNLIMITED_DEPTH: usize = usize::MAX;

/// Directory entries returned by a [`FileSystemHost`].
///
/// `files` and `directories` must be sorted for deterministic output. `symlinks` names the
/// entries in `directories` that are symlinks (so the walker re-resolves their real path);
/// `None` means the host does not track symlinks and every directory's real path is resolved
/// via [`FileSystemHost::realpath`].
#[derive(Debug, Default)]
pub struct Entries {
    pub files: Vec<String>,
    pub directories: Vec<String>,
    pub symlinks: Option<FxHashSet<String>>,
}

/// Filesystem abstraction (tsgo's `vfs.FS`, reduced to what the matcher needs).
pub trait FileSystemHost {
    fn use_case_sensitive_file_names(&self) -> bool;
    fn get_accessible_entries(&self, dir: &str) -> Entries;
    fn realpath(&self, path: &str) -> String;
}

/// Enumerate files under `path` matching `includes`/`excludes` and one of `extensions`.
pub fn read_directory(
    host: &dyn FileSystemHost,
    current_dir: &str,
    path: &str,
    extensions: &[&str],
    excludes: &[String],
    includes: &[String],
    depth: usize,
) -> Vec<String> {
    let use_cs = host.use_case_sensitive_file_names();
    let path = TsPath::from(path).normalized().into_string();
    let current_directory = TsPath::from(current_dir).normalized().into_string();
    let absolute_path =
        TsPath::from(current_directory.as_str()).combine(&[path.as_str()]).into_string();

    let file_matcher = GlobMatcher::new(includes, excludes, &absolute_path, use_cs, Usage::Files);
    let directory_matcher =
        GlobMatcher::new(includes, excludes, &absolute_path, use_cs, Usage::Directories);

    let results_len = file_matcher.includes.len().max(1);
    let mut visitor = GlobVisitor {
        host,
        file_matcher: &file_matcher,
        directory_matcher: &directory_matcher,
        extensions,
        use_case_sensitive_file_names: use_cs,
        visited: FxHashSet::default(),
        results: vec![Vec::new(); results_len],
    };

    for base_path in get_base_paths(&path, includes, use_cs) {
        let absolute =
            TsPath::from(current_directory.as_str()).combine(&[base_path.as_str()]).into_string();
        visitor.visit(&base_path, &absolute, depth, "");
    }

    if visitor.results.len() == 1 {
        return visitor.results.pop().unwrap_or_default();
    }
    visitor.results.into_iter().flatten().collect()
}

/// An `include` path `foo` is implicitly the glob `foo/**/*` when its last component has no
/// extension and no glob characters of its own.
fn is_implicit_glob(last_path_component: &str) -> bool {
    !last_path_component.contains(['.', '*', '?'])
}

fn get_include_base_path(absolute: &str) -> String {
    match absolute.find(['*', '?']) {
        None => {
            if TsPath::from(absolute).has_extension() {
                TsPath::from(absolute).directory().without_trailing_separator().to_string()
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

/// The unique non-wildcard base paths among the include patterns; the walk starts from each.
fn get_base_paths(
    path: &str,
    includes: &[String],
    use_case_sensitive_file_names: bool,
) -> Vec<String> {
    let mut base_paths = vec![path.to_string()];
    if includes.is_empty() {
        return base_paths;
    }

    let mut include_base_paths: Vec<String> = includes
        .iter()
        .map(|include| {
            let absolute = if TsPath::from(include.as_str()).is_rooted() {
                include.clone()
            } else {
                TsPath::from(path).combine(&[include.as_str()]).normalized().into_string()
            };
            get_include_base_path(&absolute)
        })
        .collect();

    include_base_paths.sort_by(|a, b| {
        if use_case_sensitive_file_names {
            a.cmp(b)
        } else {
            a.cow_to_ascii_lowercase().cmp(&b.cow_to_ascii_lowercase())
        }
    });

    for include_base_path in include_base_paths {
        let contained = base_paths.iter().any(|bp| {
            TsPath::from(bp.as_str()).contains(&include_base_path, use_case_sensitive_file_names)
        });
        if !contained {
            base_paths.push(include_base_path);
        }
    }

    base_paths
}

/// A single path segment in a glob pattern.
#[derive(Debug)]
enum Component {
    /// Exact string match (e.g. `src`).
    Literal(String),
    /// Contains `*`/`?` (e.g. `*.ts`). Include patterns skip common package folders.
    Wildcard { segments: Vec<Segment>, skip_package_folders: bool },
    /// `**` matches zero or more directories.
    DoubleAsterisk,
}

/// A piece of a wildcard component: `*.ts` becomes `[Star, Literal(".ts")]`.
#[derive(Debug)]
enum Segment {
    Literal(String),
    /// `*` matches any run of characters except `/`.
    Star,
    /// `?` matches a single character except `/`.
    Question,
}

/// A compiled glob pattern.
#[derive(Debug)]
struct GlobPattern {
    components: Vec<Component>,
    is_exclude: bool,
    case_sensitive: bool,
    /// For `Files` usage: `.min.js` is excluded unless the pattern mentions it.
    exclude_min_js: bool,
}

impl Component {
    fn parse(s: &str, is_include: bool) -> Component {
        if s == "**" {
            return Component::DoubleAsterisk;
        }
        if !s.contains(['*', '?']) {
            return Component::Literal(s.to_string());
        }
        Component::Wildcard { segments: parse_segments(s), skip_package_folders: is_include }
    }
}

fn parse_segments(s: &str) -> Vec<Segment> {
    let mut result = Vec::new();
    let mut start = 0;
    for (i, &c) in s.as_bytes().iter().enumerate() {
        if c == b'*' || c == b'?' {
            if i > start {
                result.push(Segment::Literal(s[start..i].to_string()));
            }
            result.push(if c == b'*' { Segment::Star } else { Segment::Question });
            start = i + 1;
        }
    }
    if start < s.len() {
        result.push(Segment::Literal(s[start..].to_string()));
    }
    result
}

impl GlobPattern {
    /// Compile a glob spec (e.g. `src/**/*.ts`) into a pattern, or `None` if it matches nothing.
    fn compile(
        spec: &str,
        base_path: &str,
        usage: Usage,
        case_sensitive: bool,
    ) -> Option<GlobPattern> {
        let mut parts = TsPath::from(spec).normalized_components(base_path);

        // "src/**" without a trailing filename matches nothing (for include patterns).
        if usage != Usage::Exclude && parts.last().map(String::as_str) == Some("**") {
            return None;
        }

        // Normalize root: "/home/" -> "/home".
        parts[0] = TsPath::from(parts[0].as_str()).without_trailing_separator().to_string();

        // Directories implicitly match all files: "src" -> "src/**/*".
        if is_implicit_glob(parts.last().map_or("", String::as_str)) {
            parts.push("**".to_string());
            parts.push("*".to_string());
        }

        let is_include = usage != Usage::Exclude;
        let components = parts.iter().map(|part| Component::parse(part, is_include)).collect();

        Some(GlobPattern {
            components,
            is_exclude: usage == Usage::Exclude,
            case_sensitive,
            exclude_min_js: usage == Usage::Files,
        })
    }

    fn matches(&self, path: &str) -> bool {
        self.match_path_parts(path, "", 0, 0, false)
    }

    fn matches_parts(&self, prefix: &str, suffix: &str) -> bool {
        self.match_path_parts(prefix, suffix, 0, 0, false)
    }

    fn matches_prefix_parts(&self, prefix: &str, suffix: &str) -> bool {
        self.match_path_parts(prefix, suffix, 0, 0, true)
    }

    /// Match the virtual path `prefix + suffix` against this pattern. Splitting into
    /// prefix/suffix avoids allocating a combined string on the hot call site (directory
    /// prefix ending in `/` plus a single entry name).
    fn match_path_parts(
        &self,
        prefix: &str,
        suffix: &str,
        mut path_offset: usize,
        mut comp_idx: usize,
        prefix_only: bool,
    ) -> bool {
        loop {
            let (path_part, next_offset, ok) = next_path_part_parts(prefix, suffix, path_offset);
            if !ok {
                if prefix_only {
                    return true;
                }
                return self.pattern_satisfied(comp_idx);
            }

            if comp_idx >= self.components.len() {
                return self.is_exclude && !prefix_only;
            }

            match &self.components[comp_idx] {
                Component::DoubleAsterisk => {
                    if self.match_path_parts(prefix, suffix, path_offset, comp_idx + 1, prefix_only)
                    {
                        return true;
                    }
                    if !self.is_exclude
                        && (is_hidden_path(path_part) || is_package_folder(path_part))
                    {
                        return false;
                    }
                    path_offset = next_offset;
                    continue;
                }
                Component::Literal(literal) => {
                    if !self.strings_equal(literal, path_part) {
                        return false;
                    }
                }
                Component::Wildcard { segments, skip_package_folders } => {
                    if *skip_package_folders && is_package_folder(path_part) {
                        return false;
                    }
                    if !self.match_wildcard(segments, path_part) {
                        return false;
                    }
                }
            }

            path_offset = next_offset;
            comp_idx += 1;
        }
    }

    /// The remaining components can match empty input (only trailing `**`).
    fn pattern_satisfied(&self, comp_idx: usize) -> bool {
        self.components[comp_idx..].iter().all(|c| matches!(c, Component::DoubleAsterisk))
    }

    fn match_wildcard(&self, segs: &[Segment], s: &str) -> bool {
        // Include patterns: a leading wildcard cannot match a hidden file.
        if !self.is_exclude
            && matches!(segs.first(), Some(Segment::Star | Segment::Question))
            && is_hidden_path(s)
        {
            return false;
        }

        // Fast path: single `*` followed by a literal suffix (e.g. "*.ts").
        if let [Segment::Star, Segment::Literal(suffix)] = segs {
            if s.len() < suffix.len() || !self.strings_equal(suffix, &s[s.len() - suffix.len()..]) {
                return false;
            }
            return self.should_include_min_js(s, segs);
        }

        self.match_segments(segs, s) && self.should_include_min_js(s, segs)
    }

    /// Iterative wildcard match that records only the last `*` position to avoid exponential
    /// backtracking (O(n*m)).
    fn match_segments(&self, segs: &[Segment], s: &str) -> bool {
        let s_bytes = s.as_bytes();
        let mut seg_idx = 0;
        let mut s_idx = 0;
        let mut star: Option<(usize, usize)> = None; // (segment index of `*`, string index at `*`)

        while s_idx < s.len() {
            if let Some(seg) = segs.get(seg_idx) {
                match seg {
                    Segment::Literal(lit) => {
                        let end = s_idx + lit.len();
                        if end <= s.len() && self.strings_equal(lit, &s[s_idx..end]) {
                            s_idx = end;
                            seg_idx += 1;
                            continue;
                        }
                    }
                    Segment::Question => {
                        if s_bytes[s_idx] != b'/' {
                            s_idx += utf8_char_len(s, s_idx);
                            seg_idx += 1;
                            continue;
                        }
                    }
                    Segment::Star => {
                        star = Some((seg_idx, s_idx));
                        seg_idx += 1;
                        continue;
                    }
                }
            }

            // Current segment didn't match; backtrack to the last `*` if possible.
            if let Some((star_seg, star_s)) = star
                && star_s < s.len()
                && s_bytes[star_s] != b'/'
            {
                let next = star_s + utf8_char_len(s, star_s);
                star = Some((star_seg, next));
                s_idx = next;
                seg_idx = star_seg + 1;
                continue;
            }

            return false;
        }

        // Consume any trailing stars.
        while matches!(segs.get(seg_idx), Some(Segment::Star)) {
            seg_idx += 1;
        }
        seg_idx >= segs.len()
    }

    fn should_include_min_js(&self, filename: &str, segs: &[Segment]) -> bool {
        if !self.exclude_min_js {
            return true;
        }
        if !self.has_min_js_suffix(filename) {
            return true;
        }
        // Allow when the user's pattern explicitly references the `.min.` suffix.
        self.pattern_mentions_min_suffix(segs)
    }

    fn has_min_js_suffix(&self, filename: &str) -> bool {
        const MIN_JS: &str = ".min.js";
        if self.case_sensitive {
            filename.ends_with(MIN_JS)
        } else {
            filename.len() >= MIN_JS.len()
                && filename[filename.len() - MIN_JS.len()..].eq_ignore_ascii_case(MIN_JS)
        }
    }

    fn pattern_mentions_min_suffix(&self, segs: &[Segment]) -> bool {
        segs.iter().any(|seg| {
            let Segment::Literal(lit) = seg else {
                return false;
            };
            if self.case_sensitive {
                lit.contains(".min.js") || lit.contains(".min.")
            } else {
                let lower = lit.cow_to_lowercase();
                lower.contains(".min.js") || lower.contains(".min.")
            }
        })
    }

    fn strings_equal(&self, a: &str, b: &str) -> bool {
        if self.case_sensitive { a == b } else { a.eq_ignore_ascii_case(b) }
    }
}

fn utf8_char_len(s: &str, idx: usize) -> usize {
    s[idx..].chars().next().map_or(1, char::len_utf8)
}

/// Extract the next path component from `s` starting at `offset`.
fn next_path_part_single(s: &str, mut offset: usize) -> (&str, usize, bool) {
    if offset >= s.len() {
        return ("", offset, false);
    }
    let bytes = s.as_bytes();
    if offset == 0 && bytes[0] == b'/' {
        return ("", 1, true);
    }
    while offset < s.len() && bytes[offset] == b'/' {
        offset += 1;
    }
    if offset >= s.len() {
        return ("", offset, false);
    }
    let rest = &s[offset..];
    if let Some(idx) = rest.find('/') {
        (&rest[..idx], offset + idx, true)
    } else {
        (rest, s.len(), true)
    }
}

/// Like [`next_path_part_single`] over the virtual path `prefix + suffix`.
fn next_path_part_parts<'a>(
    prefix: &'a str,
    suffix: &'a str,
    mut offset: usize,
) -> (&'a str, usize, bool) {
    if suffix.is_empty() {
        return next_path_part_single(prefix, offset);
    }
    if prefix.is_empty() {
        return next_path_part_single(suffix, offset);
    }

    // On the hot call site `prefix` is a directory path ending in '/' and `suffix` is a
    // single entry name (no '/'), so this stays simple.
    let total_len = prefix.len() + suffix.len();
    if offset >= total_len {
        return ("", offset, false);
    }

    let pbytes = prefix.as_bytes();
    if offset == 0 && pbytes[0] == b'/' {
        return ("", 1, true);
    }

    if offset < prefix.len() {
        while offset < prefix.len() && pbytes[offset] == b'/' {
            offset += 1;
        }
        if offset < prefix.len() {
            let rest = &prefix[offset..];
            // `prefix` ends in '/', so a separator is guaranteed for the hot call site.
            let idx = rest.find('/').unwrap_or(rest.len());
            return (&rest[..idx], offset + idx, true);
        }
        // Otherwise fall through into the suffix region.
    }

    let s_off = offset - prefix.len();
    if s_off >= suffix.len() {
        return ("", offset, false);
    }
    (&suffix[s_off..], total_len, true)
}

fn is_hidden_path(name: &str) -> bool {
    name.as_bytes().first() == Some(&b'.')
}

fn is_package_folder(name: &str) -> bool {
    match name.len() {
        12 => name.eq_ignore_ascii_case("node_modules"),
        13 => name.eq_ignore_ascii_case("jspm_packages"),
        16 => name.eq_ignore_ascii_case("bower_components"),
        _ => false,
    }
}

fn ensure_trailing_slash(s: &str) -> String {
    if !s.is_empty() && !s.ends_with('/') { format!("{s}/") } else { s.to_string() }
}

/// Combined include + exclude patterns.
struct GlobMatcher {
    includes: Vec<GlobPattern>,
    excludes: Vec<GlobPattern>,
    /// True if include specs were provided (even if none compiled to a usable pattern).
    had_includes: bool,
}

impl GlobMatcher {
    fn new(
        include_specs: &[String],
        exclude_specs: &[String],
        base_path: &str,
        case_sensitive: bool,
        usage: Usage,
    ) -> Self {
        let includes = include_specs
            .iter()
            .filter_map(|spec| GlobPattern::compile(spec, base_path, usage, case_sensitive))
            .collect();
        let excludes = exclude_specs
            .iter()
            .filter_map(|spec| {
                GlobPattern::compile(spec, base_path, Usage::Exclude, case_sensitive)
            })
            .collect();
        Self { includes, excludes, had_includes: !include_specs.is_empty() }
    }

    /// The index of the matching include bucket for `prefix + suffix`, or `None`.
    fn matches_file_parts(&self, prefix: &str, suffix: &str) -> Option<usize> {
        if self.excludes.iter().any(|e| e.matches_parts(prefix, suffix)) {
            return None;
        }
        if self.includes.is_empty() {
            return if self.had_includes { None } else { Some(0) };
        }
        self.includes.iter().position(|inc| inc.matches_parts(prefix, suffix))
    }

    /// Whether files under `prefix + suffix` could match any pattern (to prune the walk).
    fn matches_directory_parts(&self, prefix: &str, suffix: &str) -> bool {
        if self.excludes.iter().any(|e| e.matches_parts(prefix, suffix)) {
            return false;
        }
        if self.includes.is_empty() {
            return !self.had_includes;
        }
        self.includes.iter().any(|inc| inc.matches_prefix_parts(prefix, suffix))
    }
}

/// Walks directories collecting files that match the glob patterns.
struct GlobVisitor<'a> {
    host: &'a dyn FileSystemHost,
    file_matcher: &'a GlobMatcher,
    directory_matcher: &'a GlobMatcher,
    extensions: &'a [&'a str],
    use_case_sensitive_file_names: bool,
    visited: FxHashSet<String>,
    /// One bucket per include pattern (or a single bucket when there are no includes).
    results: Vec<Vec<String>>,
}

impl GlobVisitor<'_> {
    fn visit(
        &mut self,
        path: &str,
        absolute_path: &str,
        mut depth: usize,
        resolved_real_path: &str,
    ) {
        let host = self.host;
        let file_matcher = self.file_matcher;
        let directory_matcher = self.directory_matcher;
        let extensions = self.extensions;
        let use_cs = self.use_case_sensitive_file_names;

        // Detect symlink cycles by canonical real path.
        let real_path = if resolved_real_path.is_empty() {
            host.realpath(absolute_path)
        } else {
            resolved_real_path.to_string()
        };
        let canonical_path = TsPath::from(real_path.as_str()).canonical(use_cs);
        if !self.visited.insert(canonical_path) {
            return;
        }

        let entries = host.get_accessible_entries(absolute_path);
        let path_prefix = ensure_trailing_slash(path);
        let abs_prefix = ensure_trailing_slash(absolute_path);

        for file in &entries.files {
            if !extensions.is_empty()
                && !TsPath::from(file.as_str()).file_extension_is_one_of(extensions)
            {
                continue;
            }
            if let Some(idx) = file_matcher.matches_file_parts(&abs_prefix, file) {
                self.results[idx].push(format!("{path_prefix}{file}"));
            }
        }

        if depth != UNLIMITED_DEPTH {
            if depth == 0 {
                return;
            }
            depth -= 1;
            if depth == 0 {
                return;
            }
        }

        for dir in &entries.directories {
            if !directory_matcher.matches_directory_parts(&abs_prefix, dir) {
                continue;
            }
            let abs_dir = format!("{abs_prefix}{dir}");
            let child_real_path = match &entries.symlinks {
                // Non-symlink directory: compute the real path incrementally.
                Some(symlinks) if !symlinks.contains(dir) => {
                    TsPath::from(real_path.as_str()).combine(&[dir.as_str()]).into_string()
                }
                // Symlink directory, or the host doesn't track symlinks: force a `realpath` call.
                _ => String::new(),
            };
            self.visit(&format!("{path_prefix}{dir}"), &abs_dir, depth, &child_real_path);
        }
    }
}

/// Matches a path against one or more glob specs (used for the JSON-only include bucket).
pub struct SpecMatcher {
    patterns: Vec<GlobPattern>,
}

impl SpecMatcher {
    /// Build a matcher, or `None` if no spec compiles to a usable pattern.
    pub fn new(
        specs: &[String],
        base_path: &str,
        usage: Usage,
        use_case_sensitive_file_names: bool,
    ) -> Option<SpecMatcher> {
        if specs.is_empty() {
            return None;
        }
        let patterns: Vec<GlobPattern> = specs
            .iter()
            .filter_map(|spec| {
                GlobPattern::compile(spec, base_path, usage, use_case_sensitive_file_names)
            })
            .collect();
        if patterns.is_empty() { None } else { Some(SpecMatcher { patterns }) }
    }

    /// The index of the first matching pattern, or `None`.
    pub fn match_index(&self, path: &str) -> Option<usize> {
        self.patterns.iter().position(|p| p.matches(path))
    }
}

/// Real filesystem host backed by `std::fs`.
pub struct StdFs {
    pub use_case_sensitive_file_names: bool,
}

impl FileSystemHost for StdFs {
    fn use_case_sensitive_file_names(&self) -> bool {
        self.use_case_sensitive_file_names
    }

    fn get_accessible_entries(&self, dir: &str) -> Entries {
        let mut files = Vec::new();
        let mut directories = Vec::new();
        let mut symlinks = FxHashSet::default();

        if let Ok(read) = std::fs::read_dir(dir) {
            for entry in read.flatten() {
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                let name = entry.file_name().to_string_lossy().into_owned();
                let is_symlink = file_type.is_symlink();
                // `file_type` does not follow symlinks; classify the target via `metadata`.
                let is_dir = if is_symlink {
                    std::fs::metadata(entry.path()).is_ok_and(|m| m.is_dir())
                } else {
                    file_type.is_dir()
                };
                if is_dir {
                    directories.push(name.clone());
                    if is_symlink {
                        symlinks.insert(name);
                    }
                } else {
                    files.push(name);
                }
            }
        }

        // tsgo's vfs yields sorted entries; `read_dir` does not, so sort for determinism.
        files.sort();
        directories.sort();
        Entries { files, directories, symlinks: Some(symlinks) }
    }

    fn realpath(&self, path: &str) -> String {
        std::fs::canonicalize(path).map_or_else(
            |_| path.to_string(),
            |p| TsPath::from_slashes(&p.to_string_lossy()).into_string(),
        )
    }
}
