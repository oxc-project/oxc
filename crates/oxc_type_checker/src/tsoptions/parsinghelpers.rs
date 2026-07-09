//! Port of typescript-go's `internal/tsoptions/parsinghelpers.go` (the `extends` merge).

use rustc_hash::FxHashSet;

use crate::core::{CompilerOptions, for_each_compiler_option};

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
            $(
                if source_explicit_nulls.contains($json) {
                    target.$field = None;
                } else if source.$field.is_some() {
                    target.$field.clone_from(&source.$field);
                }
            )*
            // `paths_base_path` travels with `paths` (tsgo copies the non-zero `PathsBasePath`
            // field like any other): a config that sets `paths` also set its base path.
            if source.paths_base_path.is_some() {
                target.paths_base_path.clone_from(&source.paths_base_path);
            }
        }
    };
}
for_each_compiler_option!(define_merge_compiler_options);
