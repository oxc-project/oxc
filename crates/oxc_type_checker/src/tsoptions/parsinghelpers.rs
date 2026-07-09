//! Port of typescript-go's `internal/tsoptions/parsinghelpers.go` (the `extends` merge).

use rustc_hash::FxHashSet;

use crate::core::{CompilerOptions, for_each_compiler_option};

/// One merge statement per option, split by kind so `Copy` fields assign and the rest
/// `clone_from`.
macro_rules! merge_field {
    ($target:ident, $source:ident, $nulls:ident, $field:ident, $json:literal, bool) => {
        merge_field!(@copy $target, $source, $nulls, $field, $json);
    };
    ($target:ident, $source:ident, $nulls:ident, $field:ident, $json:literal, number) => {
        merge_field!(@copy $target, $source, $nulls, $field, $json);
    };
    ($target:ident, $source:ident, $nulls:ident, $field:ident, $json:literal, enum($ty:ty)) => {
        merge_field!(@copy $target, $source, $nulls, $field, $json);
    };
    (@copy $target:ident, $source:ident, $nulls:ident, $field:ident, $json:literal) => {
        if $nulls.contains($json) {
            $target.$field = None;
        } else if $source.$field.is_some() {
            $target.$field = $source.$field;
        }
    };
    ($target:ident, $source:ident, $nulls:ident, $field:ident, $json:literal, $kind:ident) => {
        if $nulls.contains($json) {
            $target.$field = None;
        } else if $source.$field.is_some() {
            $target.$field.clone_from(&$source.$field);
        }
    };
}

macro_rules! define_merge_compiler_options {
    ($(($field:ident, $json:literal, $($kind:tt)+)),* $(,)?) => {
        /// tsgo `mergeCompilerOptions`: copy every option that is set on `source` onto `target`
        /// (so later merges win), except that an option `source` explicitly set to JSON `null`
        /// is *reset* on `target` instead — tsc's "null clears the inherited value" rule.
        ///
        /// `source_explicit_nulls` holds the JSON names of `source`'s own explicitly-`null`
        /// options (tsgo reads them out of the config's raw map).
        pub(super) fn merge_compiler_options(
            target: &mut CompilerOptions,
            source: &CompilerOptions,
            source_explicit_nulls: &FxHashSet<&'static str>,
        ) {
            $( merge_field!(target, source, source_explicit_nulls, $field, $json, $($kind)+); )*
            // `paths_base_path` travels with `paths` (tsgo copies the non-zero `PathsBasePath`
            // field like any other): a config that sets `paths` also set its base path.
            if source.paths_base_path.is_some() {
                target.paths_base_path.clone_from(&source.paths_base_path);
            }
        }
    };
}
for_each_compiler_option!(define_merge_compiler_options);

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::core::ScriptTarget;

    #[test]
    fn set_source_options_override_and_unset_ones_inherit() {
        let mut target = CompilerOptions {
            strict: Some(false),
            target: Some(ScriptTarget::Es2015),
            ..CompilerOptions::default()
        };
        let source = CompilerOptions {
            target: Some(ScriptTarget::Es2022),
            no_emit: Some(true),
            ..CompilerOptions::default()
        };
        merge_compiler_options(&mut target, &source, &FxHashSet::default());
        // Set on source: overridden. Unset on source: target's value survives.
        assert_eq!(target.target, Some(ScriptTarget::Es2022));
        assert_eq!(target.no_emit, Some(true));
        assert_eq!(target.strict, Some(false));
    }

    #[test]
    fn explicit_null_resets_instead_of_inheriting() {
        let mut target = CompilerOptions {
            out_dir: Some(PathBuf::from("/base/dist")),
            strict: Some(true),
            ..CompilerOptions::default()
        };
        let source = CompilerOptions::default();
        let nulls = FxHashSet::from_iter(["outDir"]);
        merge_compiler_options(&mut target, &source, &nulls);
        assert_eq!(target.out_dir, None);
        assert_eq!(target.strict, Some(true));
    }

    #[test]
    fn paths_replace_wholesale_with_their_base_path() {
        let mut base_paths = crate::core::CompilerOptionsPathsMap::default();
        base_paths.insert("@base/*".to_string(), vec!["./base/*".to_string()]);
        let mut target = CompilerOptions {
            paths: Some(base_paths),
            paths_base_path: Some(PathBuf::from("/base")),
            ..CompilerOptions::default()
        };

        let mut child_paths = crate::core::CompilerOptionsPathsMap::default();
        child_paths.insert("@child/*".to_string(), vec!["./child/*".to_string()]);
        let source = CompilerOptions {
            paths: Some(child_paths),
            paths_base_path: Some(PathBuf::from("/child")),
            ..CompilerOptions::default()
        };

        merge_compiler_options(&mut target, &source, &FxHashSet::default());
        let paths = target.paths.expect("paths merged");
        // Atomic per-key: the child's `paths` object replaces the base's entirely.
        assert!(paths.contains_key("@child/*"));
        assert!(!paths.contains_key("@base/*"));
        assert_eq!(target.paths_base_path, Some(PathBuf::from("/child")));
    }
}
