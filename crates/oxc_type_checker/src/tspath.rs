//! Path utilities ported from typescript-go `internal/tspath/path.go` (POSIX + DOS + UNC
//! subset — URL/untitled roots are omitted since tsconfig specs are filesystem paths).
//!
//! tsgo's free `Path` helpers are wrapped in a [`TsPath`] newtype (mirroring Go's `type Path`)
//! so callers read as `path.directory().combine(&["tsconfig.json"])`. Trailing-separator
//! preservation and the byte-scanner fast paths tsgo keeps for other callers are dropped: the
//! matcher re-normalizes every spec, so the component-based results still match.

use std::{borrow::Cow, fmt};

use cow_utils::CowUtils;

use crate::fold;

/// Known extensions, longest declaration extensions first, used by [`TsPath::change_extension`].
const EXTENSIONS_TO_REMOVE: &[&str] = &[
    ".d.ts", ".d.mts", ".d.cts", ".mjs", ".mts", ".cjs", ".cts", ".ts", ".js", ".tsx", ".jsx",
    ".json",
];

/// A filesystem path with `/` separators; a thin newtype over `String` (tsgo's `type Path`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct TsPath(String);

impl From<&str> for TsPath {
    fn from(path: &str) -> Self {
        Self(path.to_string())
    }
}

impl From<String> for TsPath {
    fn from(path: String) -> Self {
        Self(path)
    }
}

impl fmt::Display for TsPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for TsPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TsPath {
    /// Build a path from an OS string, converting backslashes to forward slashes.
    pub fn from_slashes(path: &str) -> Self {
        Self(Self::to_slashes(path).into_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }

    /// Whether the path is rooted on a disk (POSIX `/`, DOS `c:/`, or UNC).
    pub fn is_rooted(&self) -> bool {
        Self::root_len(&self.0) != 0
    }

    pub fn has_trailing_separator(&self) -> bool {
        Self::ends_with_separator(&self.0)
    }

    pub fn without_trailing_separator(&self) -> &str {
        Self::trim_trailing_separator(&self.0)
    }

    /// Combine with more path segments. An absolute segment replaces the accumulated result;
    /// relative ones are appended. Slashes are normalized but `.`/`..` are not resolved.
    #[must_use]
    pub fn combine(&self, paths: &[&str]) -> TsPath {
        TsPath(Self::combine_into(&self.0, paths))
    }

    /// Leading portion up to (but not including) the last directory separator.
    #[must_use]
    pub fn directory(&self) -> TsPath {
        TsPath(Self::dir_of(&self.0))
    }

    /// Trailing file name after the last separator, without any trailing separator.
    pub fn base_name(&self) -> String {
        Self::base_of(&self.0)
    }

    /// Normalize `.`/`..` and slashes.
    #[must_use]
    pub fn normalized(&self) -> TsPath {
        TsPath(Self::normalized_absolute_of(&self.0, ""))
    }

    /// Resolve against `current_directory` into a normalized absolute path.
    #[must_use]
    pub fn normalized_absolute(&self, current_directory: &str) -> TsPath {
        TsPath(Self::normalized_absolute_of(&self.0, current_directory))
    }

    /// The normalized path components (root first) after resolving against `current_directory`.
    pub fn normalized_components(&self, current_directory: &str) -> Vec<String> {
        let combined = Self::combine_into(current_directory, &[&self.0]);
        Self::normalized_components_of(&combined)
    }

    /// Case-folded key for de-duplication (tsgo's `GetCanonicalFileName` /
    /// `ToFileNameLowerCase`, which lowercases everything except U+0130).
    pub fn canonical(&self, use_case_sensitive_file_names: bool) -> String {
        if use_case_sensitive_file_names {
            self.0.clone()
        } else {
            fold::to_file_name_lower_case(&self.0)
        }
    }

    pub fn file_extension_is(&self, extension: &str) -> bool {
        self.0.len() > extension.len() && self.0.ends_with(extension)
    }

    pub fn file_extension_is_one_of(&self, extensions: &[&str]) -> bool {
        extensions.iter().any(|ext| self.file_extension_is(ext))
    }

    /// Whether the base name contains a `.`.
    pub fn has_extension(&self) -> bool {
        Self::base_of(&self.0).contains('.')
    }

    /// Replace the path's known extension with `new_extension`.
    #[must_use]
    pub fn change_extension(&self, new_extension: &str) -> TsPath {
        TsPath(Self::change_any_extension(&self.0, new_extension, EXTENSIONS_TO_REMOVE))
    }

    /// Whether `child` is contained within (or equal to) this path (tsgo `ContainsPath`).
    /// Relative inputs are resolved against `current_directory` first; the volume/root
    /// component is always compared case-insensitively (Unicode fold), matching tsgo.
    pub fn contains(
        &self,
        child: &str,
        use_case_sensitive_file_names: bool,
        current_directory: &str,
    ) -> bool {
        let parent = Self::combine_into(current_directory, &[&self.0]);
        let child = Self::combine_into(current_directory, &[child]);
        if parent.is_empty() || child.is_empty() {
            return false;
        }
        if parent == child {
            return true;
        }
        let parent_components = Self::reduce_components(&Self::components_of(&parent, ""));
        let child_components = Self::reduce_components(&Self::components_of(&child, ""));
        if child_components.len() < parent_components.len() {
            return false;
        }
        for (i, parent_component) in parent_components.iter().enumerate() {
            let equal = if i == 0 || !use_case_sensitive_file_names {
                fold::str_fold_eq(parent_component, &child_components[i])
            } else {
                *parent_component == child_components[i]
            };
            if !equal {
                return false;
            }
        }
        true
    }

    /// Call `callback` on this path and each ancestor directory, returning the first `Some`.
    pub fn for_each_ancestor<T>(&self, mut callback: impl FnMut(&str) -> Option<T>) -> Option<T> {
        let mut directory = self.0.clone();
        loop {
            if let Some(result) = callback(&directory) {
                return Some(result);
            }
            let parent = Self::dir_of(&directory);
            if parent == directory {
                return None;
            }
            directory = parent;
        }
    }

    // ---- private helpers operating on raw `&str` ----

    fn is_separator(c: u8) -> bool {
        c == b'/' || c == b'\\'
    }

    fn is_volume(c: u8) -> bool {
        c.is_ascii_alphabetic()
    }

    fn to_slashes(path: &str) -> Cow<'_, str> {
        path.cow_replace('\\', "/")
    }

    fn ends_with_separator(path: &str) -> bool {
        path.as_bytes().last().is_some_and(|&c| Self::is_separator(c))
    }

    fn trim_trailing_separator(path: &str) -> &str {
        if Self::ends_with_separator(path) { &path[..path.len() - 1] } else { path }
    }

    fn ensure_separator(path: &str) -> String {
        if Self::ends_with_separator(path) { path.to_string() } else { format!("{path}/") }
    }

    /// `GetRootLength` for POSIX (`/`), UNC (`//server/`) and DOS (`c:/`) paths.
    fn root_len(path: &str) -> usize {
        let bytes = path.as_bytes();
        let ln = bytes.len();
        if ln == 0 {
            return 0;
        }
        let ch0 = bytes[0];
        if ch0 == b'/' || ch0 == b'\\' {
            if ln == 1 || bytes[1] != ch0 {
                return 1;
            }
            let offset = 2;
            return match path[offset..].bytes().position(|b| b == ch0) {
                None => ln,
                Some(p1) => p1 + offset + 1,
            };
        }
        if Self::is_volume(ch0) && ln > 1 && bytes[1] == b':' {
            if ln == 2 {
                return 2;
            }
            let ch2 = bytes[2];
            if ch2 == b'/' || ch2 == b'\\' {
                return 3;
            }
        }
        0
    }

    fn combine_into(first: &str, paths: &[&str]) -> String {
        let mut result = Self::to_slashes(first).into_owned();
        for &p in paths {
            if p.is_empty() {
                continue;
            }
            let trailing = Self::to_slashes(p);
            if result.is_empty() || Self::root_len(&trailing) != 0 {
                result = trailing.into_owned();
            } else {
                if !Self::ends_with_separator(&result) {
                    result.push('/');
                }
                result.push_str(&trailing);
            }
        }
        result
    }

    fn components_of(path: &str, current_directory: &str) -> Vec<String> {
        let path = Self::combine_into(current_directory, &[path]);
        let root_len = Self::root_len(&path);
        let mut rest: Vec<&str> = path[root_len..].split('/').collect();
        if rest.last() == Some(&"") {
            rest.pop();
        }
        let mut out = Vec::with_capacity(rest.len() + 1);
        out.push(path[..root_len].to_string());
        out.extend(rest.into_iter().map(str::to_string));
        out
    }

    fn dir_of(path: &str) -> String {
        let path = Self::to_slashes(path);
        let root_len = Self::root_len(&path);
        if root_len == path.len() {
            return path.into_owned();
        }
        let path = Self::trim_trailing_separator(&path);
        let cut = match path.rfind('/') {
            Some(i) => root_len.max(i),
            None => root_len,
        };
        path[..cut].to_string()
    }

    fn base_of(path: &str) -> String {
        let path = Self::to_slashes(path);
        let root_len = Self::root_len(&path);
        if root_len == path.len() {
            return String::new();
        }
        let path = Self::trim_trailing_separator(&path);
        let start = root_len.max(path.rfind('/').map_or(0, |i| i + 1));
        path[start..].to_string()
    }

    fn join_components(components: &[String]) -> String {
        if components.is_empty() {
            return String::new();
        }
        let root = &components[0];
        let mut result = if root.is_empty() { String::new() } else { Self::ensure_separator(root) };
        result.push_str(&components[1..].join("/"));
        result
    }

    /// Drop `.`/empty components and collapse `..` where possible (never above the root).
    fn reduce_components(components: &[String]) -> Vec<String> {
        if components.is_empty() {
            return Vec::new();
        }
        let mut reduced = vec![components[0].clone()];
        for component in &components[1..] {
            if component.is_empty() || component == "." {
                continue;
            }
            if component == ".." {
                let last_is_dotdot = reduced.last().map(String::as_str) == Some("..");
                if reduced.len() > 1 {
                    if !last_is_dotdot {
                        reduced.pop();
                        continue;
                    }
                } else if !reduced[0].is_empty() {
                    continue;
                }
            }
            reduced.push(component.clone());
        }
        reduced
    }

    fn normalized_components_of(path: &str) -> Vec<String> {
        let root_len = Self::root_len(path);
        let bytes = path.as_bytes();
        let mut components: Vec<String> = Vec::with_capacity(8);
        components.push(path[..root_len].to_string());
        let mut i = root_len;
        while i < bytes.len() {
            while i < bytes.len() && bytes[i] == b'/' {
                i += 1;
            }
            if i >= bytes.len() {
                break;
            }
            let start = i;
            while i < bytes.len() && bytes[i] != b'/' {
                i += 1;
            }
            let component = &path[start..i];
            if component.is_empty() || component == "." {
                continue;
            }
            if component == ".." {
                let last_is_dotdot = components.last().map(String::as_str) == Some("..");
                if components.len() > 1 {
                    if !last_is_dotdot {
                        components.pop();
                        continue;
                    }
                } else if !components[0].is_empty() {
                    continue;
                }
            }
            components.push(component.to_string());
        }
        components
    }

    fn normalized_absolute_of(file_name: &str, current_directory: &str) -> String {
        let combined = if Self::root_len(file_name) == 0 && !current_directory.is_empty() {
            Self::combine_into(current_directory, &[file_name])
        } else {
            Self::to_slashes(file_name).into_owned()
        };
        Self::join_components(&Self::normalized_components_of(&combined))
    }

    fn any_extension(path: &str, extensions: &[&str]) -> String {
        let path = Self::trim_trailing_separator(path);
        for &extension in extensions {
            // Every candidate extension starts with '.'.
            if path.len() >= extension.len() {
                let idx = path.len() - extension.len();
                if path.as_bytes()[idx] == b'.' && &path[idx..] == extension {
                    return extension.to_string();
                }
            }
        }
        String::new()
    }

    fn change_any_extension(path: &str, new_extension: &str, extensions: &[&str]) -> String {
        let pathext = Self::any_extension(path, extensions);
        if pathext.is_empty() {
            return path.to_string();
        }
        let result = &path[..path.len() - pathext.len()];
        if new_extension.is_empty() {
            result.to_string()
        } else if new_extension.starts_with('.') {
            format!("{result}{new_extension}")
        } else {
            format!("{result}.{new_extension}")
        }
    }
}
