use std::path::{Component, Path, PathBuf};

use cow_utils::CowUtils;
use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{Span, VALID_EXTENSIONS};
use serde_json::Value;

use crate::{AstNode, ModuleRecord, context::LintContext, rule::Rule};

fn no_useless_path_segments_diagnostic(
    span: Span,
    import_path: &str,
    proposed: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Useless path segments for \"{import_path}\", should be \"{proposed}\""
    ))
    .with_label(span)
}

/// <https://github.com/import-js/eslint-plugin-import/blob/v2.31.0/docs/rules/no-useless-path-segments.md>
#[derive(Default, Debug, Clone)]
pub struct NoUselessPathSegments {
    commonjs: bool,
    no_useless_index: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents unnecessary path segments in import and require statements.
    ///
    /// ### Why is this bad?
    ///
    /// Useless path segments like `./`, `../`, or redundant directory navigation
    /// make imports harder to read and maintain.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import foo from './../bar';      // should be '../bar'
    /// import bar from './foo/../bar';  // should be './bar'
    /// import baz from './';            // should be '.'
    /// import qux from './deep//a';     // should be './deep/a'
    ///
    /// // With noUselessIndex option
    /// import x from './foo/index.js';  // should be './foo' or './foo/'
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// import foo from '../bar';
    /// import bar from './bar';
    /// import baz from '.';
    /// import qux from './deep/a';
    /// import external from 'lodash';
    /// ```
    NoUselessPathSegments,
    import,
    pedantic
);

impl Rule for NoUselessPathSegments {
    fn from_configuration(value: Value) -> Self {
        let obj = value.get(0);
        Self {
            commonjs: obj.and_then(|v| v.get("commonjs")).and_then(Value::as_bool).unwrap_or(false),
            no_useless_index: obj
                .and_then(|v| v.get("noUselessIndex"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let module_record = ctx.module_record();

        match node.kind() {
            // Check dynamic import() expressions
            AstKind::ImportExpression(import_expr) => {
                let import_path = match &import_expr.source {
                    Expression::StringLiteral(str_lit) => &str_lit.value,
                    Expression::TemplateLiteral(tpl) if tpl.is_no_substitution_template() => {
                        if let Some(quasi) = tpl.quasis.first() {
                            &quasi.value.raw
                        } else {
                            return;
                        }
                    }
                    _ => return,
                };

                if import_path.starts_with('.')
                    && let Some(proposed) =
                        self.check_path(import_path, ctx.file_path(), module_record, false, None)
                {
                    ctx.diagnostic(no_useless_path_segments_diagnostic(
                        import_expr.span,
                        import_path,
                        &proposed,
                    ));
                }
            }
            // Check CommonJS require() calls if enabled
            AstKind::CallExpression(call_expr) if self.commonjs => {
                if call_expr.callee.is_specific_id("require")
                    && call_expr.arguments.len() == 1
                    && let Some(arg) =
                        call_expr.arguments.first().and_then(|arg| arg.as_expression())
                {
                    let import_path = match arg {
                        Expression::StringLiteral(str_lit) => &str_lit.value,
                        Expression::TemplateLiteral(tpl) if tpl.is_no_substitution_template() => {
                            if let Some(quasi) = tpl.quasis.first() {
                                &quasi.value.raw
                            } else {
                                return;
                            }
                        }
                        _ => return,
                    };

                    if import_path.starts_with('.') {
                        // Compute canonical path on-demand for CommonJS self-import checks.
                        // This is only computed when needed since it's an expensive operation.
                        let current_file_canonical = ctx.file_path().canonicalize().ok();

                        if let Some(proposed) = self.check_path(
                            import_path,
                            ctx.file_path(),
                            module_record,
                            true,
                            current_file_canonical.as_ref(),
                        ) {
                            ctx.diagnostic(no_useless_path_segments_diagnostic(
                                call_expr.span,
                                import_path,
                                &proposed,
                            ));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();

        // Check ESM imports/exports via module_record (faster, uses pre-resolved data)
        for (import_path, requested_modules) in &module_record.requested_modules {
            if !import_path.starts_with('.') {
                continue;
            }

            for requested_module in requested_modules {
                if let Some(proposed) =
                    self.check_path(import_path, ctx.file_path(), module_record, false, None)
                {
                    ctx.diagnostic(no_useless_path_segments_diagnostic(
                        requested_module.span,
                        import_path,
                        &proposed,
                    ));
                }
            }
        }
    }
}

impl NoUselessPathSegments {
    /// Check if a path component is an index file.
    ///
    /// Returns `true` for "index" or "index.{ext}" where ext is in VALID_EXTENSIONS.
    /// Avoids string allocations by using string slicing.
    ///
    /// # Examples
    ///
    /// assert!(is_index_filename("index"));
    /// assert!(is_index_filename("index.js"));
    /// assert!(is_index_filename("index.ts"));
    /// assert!(!is_index_filename("index.foo"));
    /// assert!(!is_index_filename("notindex"));
    fn is_index_filename(name: &str) -> bool {
        name == "index" || Self::is_index_with_extension(name)
    }

    /// Check if name is "index.{ext}" where ext is a valid JavaScript/TypeScript extension.
    #[inline]
    fn is_index_with_extension(name: &str) -> bool {
        name.strip_prefix("index.").is_some_and(|ext| VALID_EXTENSIONS.contains(&ext))
    }

    /// Check if a resolved path points to an index file.
    ///
    /// This indicates that the import resolved to a directory,
    /// which means we might be able to suggest removing the "/index" suffix.
    ///
    /// # Examples
    ///
    /// /path/to/bar/index.js → true (can suggest "./bar")
    /// /path/to/foo.js → false (explicit file, can't simplify)
    fn resolved_to_index_file(resolved_path: &Path) -> bool {
        resolved_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(Self::is_index_filename)
    }

    /// Check if a file exists with JavaScript/TypeScript extensions.
    ///
    /// For ESM imports, this first checks module_record (fast).
    /// For CommonJS, this falls back to filesystem checks since module_record
    /// doesn't track require() calls.
    ///
    /// This is only used as a fallback for ambiguous cases.
    fn file_exists_with_extensions(
        path: &str,
        current_file: &Path,
        module_record: &ModuleRecord,
        is_require: bool,
    ) -> bool {
        // For ESM: prefer module_record lookup (much faster)
        if !is_require && module_record.get_loaded_module(path).is_some() {
            return true;
        }

        // For CommonJS or when not in module_record: check filesystem
        let Some(current_dir) = current_file.parent() else {
            return false;
        };

        let joined = current_dir.join(path);

        // Check if file exists as-is
        if joined.is_file() {
            return true;
        }

        // Try with each valid extension
        for ext in VALID_EXTENSIONS {
            if joined.with_extension(ext).is_file() {
                return true;
            }
        }

        false
    }

    /// Helper to optionally remove useless index if the feature is enabled.
    /// Reduces code duplication by wrapping the common pattern.
    fn maybe_remove_index(
        &self,
        path: String,
        module_record: &ModuleRecord,
        is_require: bool,
    ) -> String {
        if self.no_useless_index {
            Self::remove_useless_index(&path, module_record, is_require).unwrap_or(path)
        } else {
            path
        }
    }

    /// Check if a trailing slash is meaningful (i.e., changes resolution).
    ///
    /// Trailing slashes can be meaningful when:
    /// - `./bar/` resolves to `bar/index.js` (directory)
    /// - `./bar` resolves to `bar.js` (file)
    ///
    /// Returns `true` if the slash should be kept, `false` if it can be removed.
    fn is_trailing_slash_meaningful(
        import_path: &str,
        without_slash: &str,
        current_file: &Path,
        module_record: &ModuleRecord,
        is_require: bool,
    ) -> bool {
        let with_slash = module_record.get_loaded_module(import_path);
        let without = module_record.get_loaded_module(without_slash);

        match (with_slash, without) {
            (Some(slashed), Some(unslashed)) => {
                // Both exist - slash is meaningful if they resolve to different files
                slashed.resolved_absolute_path != unslashed.resolved_absolute_path
            }
            (Some(module), None) | (None, Some(module)) => {
                // Only one variant exists - slash is meaningful if:
                // 1. It resolved to a directory (index file), AND
                // 2. There's a sibling file that the other variant would resolve to
                Self::resolved_to_index_file(&module.resolved_absolute_path)
                    && Self::file_exists_with_extensions(
                        without_slash,
                        current_file,
                        module_record,
                        is_require,
                    )
            }
            (None, None) => {
                // Neither in module_record - slash is meaningful if sibling file exists
                Self::file_exists_with_extensions(
                    without_slash,
                    current_file,
                    module_record,
                    is_require,
                )
            }
        }
    }

    /// Check CommonJS require() for self-import simplification.
    ///
    /// This handles cases where a require() call imports the current file
    /// through a complex relative path. For example:
    ///
    /// ```javascript
    /// // In files/foo.js:
    /// require('../files/foo')  // → suggest './foo'
    /// ```
    ///
    /// Note: This uses filesystem operations since module_record doesn't track
    /// CommonJS require() calls.
    ///
    /// # Parameters
    ///
    /// * `current_file_canonical` - Pre-computed canonical path, cached in `run_once`
    ///   to avoid expensive repeated filesystem calls for each require().
    fn check_commonjs_self_import(
        &self,
        import_path: &str,
        current_file: &Path,
        module_record: &ModuleRecord,
        is_require: bool,
        current_file_canonical: Option<&PathBuf>,
    ) -> Option<String> {
        let current_dir = current_file.parent()?;

        let resolved = current_dir.join(import_path);

        // Try to resolve to an actual file using VALID_EXTENSIONS
        let resolved_file = if resolved.is_file() {
            Some(resolved)
        } else {
            VALID_EXTENSIONS.iter().find_map(|ext| {
                let with_ext = resolved.with_extension(ext);
                if with_ext.is_file() { Some(with_ext) } else { None }
            })
        };

        let resolved_file = resolved_file?;

        // Use the cached canonical path (computed once in run_once)
        // This avoids expensive repeated canonicalize() calls for each require()
        let current_canonical = current_file_canonical?;

        // Check if it resolves to the current file (self-import)
        if resolved_file.canonicalize().ok().as_ref() != Some(current_canonical) {
            return None;
        }

        // Build the simplest path: ./filename or ./filename.ext
        let file_name = current_file.file_stem()?.to_str()?;
        let extension = current_file.extension().and_then(|e| e.to_str());

        let suggested = if let Some(ext) = extension {
            // Preserve extension only if original import had it
            if import_path.ends_with(&format!(".{ext}")) {
                format!("./{file_name}.{ext}")
            } else {
                format!("./{file_name}")
            }
        } else {
            format!("./{file_name}")
        };

        Some(self.maybe_remove_index(suggested, module_record, is_require))
    }

    /// Main entry point for checking an import path for useless segments.
    ///
    /// This follows the original JavaScript implementation's checking order:
    /// 1. Textual normalization (e.g., "./foo/../bar" → "./bar")
    /// 2. Useless index removal (e.g., "./bar/index.js" → "./bar")
    /// 3. Early return if path is already optimal (starts with "./")
    /// 4. Resolution-based simplification (e.g., "../files/foo" → "./foo")
    ///
    /// # Parameters
    ///
    /// * `current_file_canonical` - Cached canonical path of the current file,
    ///   computed once in `run_once` to avoid repeated expensive filesystem calls.
    fn check_path(
        &self,
        import_path: &str,
        current_file: &Path,
        module_record: &ModuleRecord,
        is_require: bool,
        current_file_canonical: Option<&PathBuf>,
    ) -> Option<String> {
        // Step 1: Try textual normalization
        if let Some(normalized) = self.try_normalize(
            import_path,
            current_file,
            module_record,
            is_require,
            current_file_canonical,
        ) {
            return Some(normalized);
        }

        // Step 2: Try removing useless index (if enabled and normalization didn't trigger)
        if self.no_useless_index
            && let Some(without_index) =
                Self::remove_useless_index(import_path, module_record, is_require)
        {
            return Some(without_index);
        }

        // Step 3: Early return for already-optimal paths
        // If path starts with "./" and is already normalized, it's optimal
        if import_path.starts_with("./") {
            return None;
        }

        // Step 4: Try resolution-based simplification
        self.try_simplify_by_resolution(
            import_path,
            current_file,
            module_record,
            is_require,
            current_file_canonical,
        )
    }

    /// Try to normalize the path textually (e.g., "./foo/../bar" → "./bar").
    ///
    /// This also handles special cases like trailing slashes and checks whether
    /// normalization would break resolution.
    fn try_normalize(
        &self,
        import_path: &str,
        current_file: &Path,
        module_record: &ModuleRecord,
        is_require: bool,
        _current_file_canonical: Option<&PathBuf>,
    ) -> Option<String> {
        let normalized = normalize_path(import_path);

        if normalized == import_path {
            return None; // Already normalized
        }

        // Special handling for trailing slashes
        // Trailing slashes can be meaningful:
        // - "./bar/" → bar/index.js (directory)
        // - "./bar" → bar.js (file)
        if import_path.ends_with('/') && !normalized.ends_with('/') {
            let without_slash = import_path.trim_end_matches('/');
            if normalized == without_slash {
                // For CommonJS: suggest removing trailing slash
                // (module_record doesn't track require() calls, so we can't check resolution)
                if is_require {
                    return Some(self.maybe_remove_index(normalized, module_record, is_require));
                }

                // For ESM: Check if trailing slash is meaningful
                // (e.g., ./bar.js and ./bar/index.js both exist)
                if Self::is_trailing_slash_meaningful(
                    import_path,
                    without_slash,
                    current_file,
                    module_record,
                    is_require,
                ) {
                    return None; // Slash is meaningful, don't suggest removal
                }

                // Safe to suggest removing trailing slash
                return Some(self.maybe_remove_index(normalized, module_record, is_require));
            }
        }

        // For CommonJS, suggest normalized version without resolution checks
        // (module_record only tracks ESM imports, not require() calls)
        if is_require {
            return Some(self.maybe_remove_index(normalized, module_record, is_require));
        }

        // For ESM, check if normalization would break resolution
        // Default to suggesting normalization unless we can PROVE it breaks
        let original_module = module_record.get_loaded_module(import_path);
        let normalized_module = module_record.get_loaded_module(&normalized);

        // Only skip if both are in module_record AND resolve to different files
        if let (Some(orig), Some(norm)) = (original_module, normalized_module)
            && orig.resolved_absolute_path != norm.resolved_absolute_path
        {
            // They resolve to different files - normalization would break it
            return None;
        }

        // Otherwise, suggest normalization
        Some(self.maybe_remove_index(normalized, module_record, is_require))
    }

    /// Try resolution-based simplification.
    ///
    /// This tries to find simpler paths that resolve to the same module.
    /// For example, from a file at "files/foo.js", importing "../files/foo"
    /// can be simplified to "./foo".
    ///
    /// We iterate through progressively simpler paths rather than calculating
    /// directly (like the JS implementation) because module_record may not have
    /// complete resolution info for all cases.
    fn try_simplify_by_resolution(
        &self,
        import_path: &str,
        current_file: &Path,
        module_record: &ModuleRecord,
        is_require: bool,
        current_file_canonical: Option<&PathBuf>,
    ) -> Option<String> {
        // For CommonJS: use targeted filesystem checks since module_record doesn't track require()
        if is_require {
            return self.check_commonjs_self_import(
                import_path,
                current_file,
                module_record,
                is_require,
                current_file_canonical,
            );
        }

        // For ESM: use module_record exclusively (no filesystem checks needed)
        let original_module = module_record.get_loaded_module(import_path)?;

        // Special case: Check if the import resolves to the current file (self-import)
        // E.g., from foo.js, import '../files/foo.js' → suggests './foo.js'
        if original_module.resolved_absolute_path == module_record.resolved_absolute_path {
            return self.suggest_self_import_path(
                import_path,
                current_file,
                module_record,
                is_require,
            );
        }

        // General case: Try progressively simpler paths
        self.try_remove_parent_directories(
            import_path,
            original_module.resolved_absolute_path.as_path(),
            module_record,
            is_require,
        )
    }

    /// Suggest the shortest path for a self-import.
    ///
    /// When a module imports itself, the shortest path is always "./filename"
    /// (with extension if the original import had one).
    fn suggest_self_import_path(
        &self,
        import_path: &str,
        current_file: &Path,
        module_record: &ModuleRecord,
        is_require: bool,
    ) -> Option<String> {
        let file_name = current_file.file_stem()?.to_str()?;
        let extension = current_file.extension().and_then(|e| e.to_str());

        let suggested = if let Some(ext) = extension {
            // Preserve extension only if original import had it
            if import_path.ends_with(&format!(".{ext}")) {
                format!("./{file_name}.{ext}")
            } else {
                format!("./{file_name}")
            }
        } else {
            format!("./{file_name}")
        };

        Some(self.maybe_remove_index(suggested, module_record, is_require))
    }

    /// Try removing unnecessary parent directory traversals.
    ///
    /// For paths with ".." segments, try progressively removing them to find
    /// simpler paths that resolve to the same location.
    fn try_remove_parent_directories(
        &self,
        import_path: &str,
        resolved_path: &Path,
        module_record: &ModuleRecord,
        is_require: bool,
    ) -> Option<String> {
        let original_parents = import_path.matches("..").count();
        if original_parents == 0 {
            return None;
        }

        let import_segments: Vec<&str> = import_path.split('/').collect();

        for parents_to_remove in 1..=original_parents {
            let remaining_parents = original_parents - parents_to_remove;

            let mut new_segments: Vec<&str> =
                std::iter::repeat_n("..", remaining_parents).collect();

            for seg in import_segments.iter().skip(original_parents + parents_to_remove) {
                new_segments.push(*seg);
            }

            let suggested = if new_segments.is_empty() {
                ".".to_string()
            } else {
                let joined = new_segments.join("/");
                if joined.starts_with("..") { joined } else { format!("./{joined}") }
            };

            if let Some(suggested_module) = module_record.get_loaded_module(&suggested)
                && suggested_module.resolved_absolute_path == resolved_path
            {
                return Some(self.maybe_remove_index(suggested, module_record, is_require));
            }
        }

        None
    }

    /// Remove useless "/index" or "/index.{ext}" suffixes from import paths.
    ///
    /// When `noUselessIndex` is enabled, paths like "./bar/index.js" can be
    /// simplified to "./bar" since Node.js resolution handles this automatically.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// import './bar/index.js'  // → './bar'
    /// import './bar/index'     // → './bar'
    /// import './index'         // → '.'
    /// import '../index.ts'     // → '..'
    /// ```
    fn remove_useless_index(
        import_path: &str,
        module_record: &ModuleRecord,
        is_require: bool,
    ) -> Option<String> {
        // Check if path ends with /index or /index.{ext} where ext is in VALID_EXTENSIONS
        // This matches the resolver's behavior which uses the same VALID_EXTENSIONS
        let has_index_suffix = import_path.ends_with("/index")
            || VALID_EXTENSIONS.iter().any(|ext| import_path.ends_with(&format!("/index.{ext}")));

        if !has_index_suffix {
            // Check bare "index" cases: ./index, ../index, ./index.ext, ../index.ext
            return Self::try_remove_bare_index(import_path, module_record, is_require);
        }

        let parent = import_path.rsplit_once('/').map(|(p, _)| p)?;
        let parent = if parent.is_empty() { "." } else { parent };

        // For CommonJS, suggest removing /index without resolution checks
        if is_require {
            return Some(parent.to_string());
        }

        // For ESM, check resolution if possible
        // If the /index path resolves, suggest removing it
        // The resolver will handle ./bar the same as ./bar/index.js
        if module_record.get_loaded_module(import_path).is_some() {
            // Check if we can verify the parent also resolves
            let without_index = module_record.get_loaded_module(parent);
            let with_trailing_slash = module_record.get_loaded_module(&format!("{parent}/"));

            match (without_index, with_trailing_slash) {
                // Parent resolves - suggest it
                (Some(_), _) | (None, None) => Some(parent.to_string()),
                // Parent/ resolves - suggest it
                (_, Some(_)) => Some(format!("{parent}/")),
            }
        } else {
            // /index path doesn't resolve, don't suggest anything
            None
        }
    }

    /// Try to remove bare "index" references (e.g., "./index" → ".").
    ///
    /// Handles special cases where the import is directly to an index file
    /// without a parent directory in the path.
    fn try_remove_bare_index(
        import_path: &str,
        module_record: &ModuleRecord,
        is_require: bool,
    ) -> Option<String> {
        // Check if this is a bare index import
        let is_bare_index = import_path == "./index"
            || import_path == "../index"
            || VALID_EXTENSIONS.iter().any(|ext| {
                import_path == format!("./index.{ext}") || import_path == format!("../index.{ext}")
            });

        if !is_bare_index {
            return None;
        }

        let parent = if import_path.starts_with("./") { "." } else { ".." };

        // For CommonJS, do textual check only
        if is_require {
            return Some(parent.to_string());
        }

        // For ESM, check if parent resolves to same location
        if let (Some(index_mod), Some(parent_mod)) =
            (module_record.get_loaded_module(import_path), module_record.get_loaded_module(parent))
            && index_mod.resolved_absolute_path == parent_mod.resolved_absolute_path
        {
            return Some(parent.to_string());
        }

        None
    }
}

/// Normalize a relative path by resolving "." and ".." components.
///
/// This performs textual normalization without filesystem access:
/// - Removes redundant "./" segments (except leading one)
/// - Resolves "../" against preceding path segments
/// - Removes double slashes
/// - Removes trailing slashes
///
/// # Examples
///
/// normalize_path("./foo/../bar") → "./bar"
/// normalize_path("./deep//a")    → "./deep/a"
/// normalize_path("./../foo")     → "../foo"
/// normalize_path("./")           → "."
/// normalize_path("./bar/")       → "./bar"
fn normalize_path(path: &str) -> String {
    let path_buf = PathBuf::from(path);
    let mut components = Vec::new();

    for component in path_buf.components() {
        match component {
            Component::CurDir => {
                // Only keep initial "." if it's the first component
                // This preserves the relative nature of the path (e.g., "./foo")
                if components.is_empty() {
                    components.push(component);
                }
                // Otherwise skip redundant "." (e.g., "./foo/./bar" → "./foo/bar")
            }
            Component::ParentDir => {
                // Try to resolve ".." against preceding components
                if let Some(Component::Normal(_)) = components.last() {
                    // Can go up: "./foo/../bar" → "./bar"
                    components.pop();
                } else if matches!(components.last(), Some(Component::CurDir)) {
                    // Convert "./.." to just ".."
                    components.pop();
                    components.push(component);
                } else {
                    // Already at ".." or empty, keep adding ".."
                    // This handles cases like "../../foo"
                    components.push(component);
                }
            }
            _ => components.push(component),
        }
    }

    let result =
        components.iter().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/");

    // Clean up double slashes and trailing slashes
    let result = result.cow_replace("//", "/");
    let trimmed = result.trim_end_matches('/');

    // Return the trimmed result, but preserve ".", "..", or empty
    // (trailing slashes are handled separately by check_path with resolution checks)
    trimmed.to_string()
}

#[test]
fn test_normalize_path() {
    assert_eq!(normalize_path("./deep//a"), "./deep/a");
    assert_eq!(normalize_path("./../foo"), "../foo");
    assert_eq!(normalize_path("./foo/./bar"), "./foo/bar");
    assert_eq!(normalize_path("./"), ".");
    assert_eq!(normalize_path("../"), "..");
    assert_eq!(normalize_path("./bar/"), "./bar"); // Remove trailing slash
    assert_eq!(normalize_path("./test-module/"), "./test-module"); // Remove trailing slash
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        // CommonJS modules with default options
        ("require('./../files/malformed.js')", None),
        // ES modules with default options
        ("import './malformed.js'", None),
        ("import './test-module'", None),
        ("import './bar/'", None),
        ("import '.'", None),
        ("import '..'", None),
        ("import fs from 'fs'", None),
        // ES modules + noUselessIndex
        ("import '../index'", None), // noUselessIndex is false by default
        ("import '../my-custom-index'", Some(json!([{"noUselessIndex": true}]))),
        ("import './bar.js'", Some(json!([{"noUselessIndex": true}]))), // ./bar/index.js exists
        ("import './bar'", Some(json!([{"noUselessIndex": true}]))),
        ("import './bar/'", Some(json!([{"noUselessIndex": true}]))), // ./bar.js exists
        ("import './malformed.js'", Some(json!([{"noUselessIndex": true}]))), // ./malformed directory does not exist
        ("import './malformed'", Some(json!([{"noUselessIndex": true}]))), // ./malformed directory does not exist
        ("import './importType'", Some(json!([{"noUselessIndex": true}]))), // ./importType.js does not exist
        ("import('.')", None),
        ("import('..')", None),
        ("import('fs').then(function(fs) {})", None),
    ];

    let fail = vec![
        // CommonJS modules
        ("const foo = require('./../files/malformed.js')", Some(json!([{"commonjs": true}]))),
        ("const foo = require('./../files/malformed')", Some(json!([{"commonjs": true}]))),
        ("const foo = require('./test-module/')", Some(json!([{"commonjs": true}]))),
        ("const foo = require('./')", Some(json!([{"commonjs": true}]))),
        ("const foo = require('../')", Some(json!([{"commonjs": true}]))),
        ("const foo = require('./deep//a')", Some(json!([{"commonjs": true}]))),
        // CommonJS modules + noUselessIndex
        (
            "const foo = require('./bar/index.js')",
            Some(json!([{"commonjs": true, "noUselessIndex": true}])),
        ),
        (
            "const foo = require('./bar/index')",
            Some(json!([{"commonjs": true, "noUselessIndex": true}])),
        ),
        (
            "const foo = require('./importType/index')",
            Some(json!([{"commonjs": true, "noUselessIndex": true}])),
        ),
        (
            "const foo = require('./index')",
            Some(json!([{"commonjs": true, "noUselessIndex": true}])),
        ),
        (
            "const foo = require('../index')",
            Some(json!([{"commonjs": true, "noUselessIndex": true}])),
        ),
        (
            "const foo = require('../index.js')",
            Some(json!([{"commonjs": true, "noUselessIndex": true}])),
        ),
        // ES modules
        ("import './../files/malformed.js'", None),
        ("import './../files/malformed'", None),
        ("import './test-module/'", None),
        ("import './'", None),
        ("import '../'", None),
        ("import './deep//a'", None),
        // ES modules + noUselessIndex
        ("import './bar/index.js'", Some(json!([{"noUselessIndex": true}]))), // ./bar.js exists
        ("import './bar/index'", Some(json!([{"noUselessIndex": true}]))),    // ./bar.js exists
        ("import './index'", Some(json!([{"noUselessIndex": true}]))),
        ("import '../index'", Some(json!([{"noUselessIndex": true}]))),
        ("import '../index.js'", Some(json!([{"noUselessIndex": true}]))),
        ("import('./')", None),
        ("import('../')", None),
        ("import('./deep//a')", None),
    ];

    Tester::new(NoUselessPathSegments::NAME, NoUselessPathSegments::PLUGIN, pass, fail)
        .with_import_plugin(true)
        .change_rule_path("malformed.js")
        .test_and_snapshot();
}
