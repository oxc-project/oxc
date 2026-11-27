use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_useless_path_segments_diagnostic(
    import_path: &str,
    proposed_path: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Useless path segments for \"{import_path}\", should be \"{proposed_path}\""
    ))
    .with_help("Remove unnecessary path segments from the import path.")
    .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoUselessPathSegments {
    /// When `true`, CommonJS `require()` calls are also checked.
    commonjs: bool,
    /// When `true`, imports pointing to an `index` file (`./foo/index`, `./foo/index.js`)
    /// are reported and shortened to their directory (`./foo`).
    no_useless_index: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbids unnecessary path segments in import and require statements, such as
    /// redundant `..` hops, `.` segments, duplicate slashes, and trailing slashes.
    /// With the `noUselessIndex` option it also reports explicit imports of a
    /// directory's `index` file.
    ///
    /// ### Why is this bad?
    ///
    /// Import paths with useless segments are longer than necessary and obscure
    /// where the imported module actually lives. Normalizing them keeps import
    /// paths consistent and easier to read, and avoids pointless churn when files
    /// move.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule (from `my-project/app.js`):
    /// ```js
    /// import "./../my-project/pages/about.js"; // should be "./pages/about.js"
    /// import "./pages//about"; // should be "./pages/about"
    /// import "./pages/"; // should be "./pages"
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import "./pages/about";
    /// import ".";
    /// import "..";
    /// import "fs";
    /// ```
    ///
    /// ### Known divergences from the original rule
    ///
    /// This port is purely lexical and never touches the filesystem, while the
    /// original consults a resolver in three places. As a result:
    ///
    /// - A path whose only abnormality is a trailing slash (`./foo/`) is always
    ///   reported, even in the rare case where both `foo/index.js` and `foo.js`
    ///   exist and the original keeps the trailing slash. Because removing the
    ///   slash can switch resolution from the directory `index` to the sibling
    ///   file, these fixes are marked dangerous.
    /// - With `noUselessIndex`, `./foo/index.js` is always shortened to `./foo`;
    ///   the original proposes `./foo/` when a sibling `foo.js` makes `./foo`
    ///   ambiguous. These fixes are marked dangerous as well.
    /// - Excess `..` segments are reported even when the import does not resolve
    ///   (the original stays silent on unresolved imports).
    ///
    /// ### References
    ///
    /// Ported from [eslint-plugin-import/no-useless-path-segments](https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/no-useless-path-segments.md).
    NoUselessPathSegments,
    import,
    style,
    conditional_fix_dangerous,
    config = NoUselessPathSegments,
    version = "next",
    short_description = "Forbid unnecessary path segments in import and require statements.",
);

impl Rule for NoUselessPathSegments {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            // ESM import declarations: `import x from '...'`, `import '...'`
            AstKind::ImportDeclaration(import_decl) => {
                self.check_source(
                    import_decl.source.value.as_str(),
                    import_decl.source.span,
                    false,
                    ctx,
                );
            }
            // ESM re-exports: `export { x } from '...'`
            AstKind::ExportFromDeclaration(export_decl) => {
                self.check_source(
                    export_decl.source.value.as_str(),
                    export_decl.source.span,
                    false,
                    ctx,
                );
            }
            // ESM star re-exports: `export * from '...'`
            AstKind::ExportAllDeclaration(export_decl) => {
                self.check_source(
                    export_decl.source.value.as_str(),
                    export_decl.source.span,
                    false,
                    ctx,
                );
            }
            // Dynamic imports: `import('...')`
            AstKind::ImportExpression(import_expr) => {
                if let Expression::StringLiteral(str_literal) = &import_expr.source {
                    self.check_source(str_literal.value.as_str(), str_literal.span, false, ctx);
                }
            }
            // CommonJS `require('...')` (only with the `commonjs` option)
            AstKind::CallExpression(call_expr) if self.commonjs => {
                if let Some(str_literal) = call_expr.common_js_require() {
                    self.check_source(str_literal.value.as_str(), str_literal.span, false, ctx);
                } else if call_expr.callee.is_specific_id("require")
                    && call_expr.arguments.len() == 1
                {
                    // `require(`...`)` with a substitution-free template literal
                    if let Argument::TemplateLiteral(template) = &call_expr.arguments[0]
                        && template.expressions.is_empty()
                        && template.quasis.len() == 1
                        && let Some(cooked) = &template.quasis[0].value.cooked
                    {
                        self.check_source(cooked.as_str(), template.span, true, ctx);
                    }
                }
            }
            _ => {}
        }
    }
}

impl NoUselessPathSegments {
    fn check_source(
        &self,
        import_path: Option<&str>,
        span: Span,
        is_template: bool,
        ctx: &LintContext<'_>,
    ) {
        let Some(import_path) = import_path else { return };

        // Only relative imports are relevant for this rule
        if !import_path.starts_with('.') {
            return;
        }

        // Report if the path is not in normal form (redundant `.` / `..` hops, `//`,
        // trailing `/`). Collapsing `.` / `..` / `//` segments is purely lexical and
        // cannot change what the path resolves to. Removing a trailing slash can:
        // `./foo/` always targets the directory, while `./foo` resolves to a sibling
        // `foo.js` when one exists, so those fixes are marked dangerous. The
        // exception is a proposal of `.` or `..`, which keeps directory resolution.
        if !is_normalized(import_path) {
            let normalized = normalize_relative(import_path);
            let dangerous = import_path.ends_with('/') && normalized != "." && normalized != "..";
            Self::report(import_path, &normalized, span, is_template, dangerous, ctx);
            return;
        }

        // Check for an unnecessary `index` (including an extension from
        // `settings.import.extensions`, mirroring eslint-plugin-import's
        // `import/extensions` setting).
        if self.no_useless_index && has_index_suffix(import_path, &ctx.settings().import.extensions)
        {
            // `rfind` cannot fail: `has_index_suffix` requires a `/`
            let parent_directory = &import_path[..import_path.rfind('/').unwrap()];
            // `./foo` resolves to a sibling `foo.js` when one exists, while the
            // original `./foo/index.js` always targets the directory, so the fix is
            // dangerous unless the proposal keeps directory resolution (`.` / `..`).
            let dangerous = parent_directory != "." && parent_directory != "..";
            Self::report(import_path, parent_directory, span, is_template, dangerous, ctx);
            return;
        }

        // Paths starting from the current directory cannot have too many `..` segments
        if import_path.starts_with("./") {
            return;
        }

        let Some(current_dir) = ctx.file_path().parent() else { return };

        // Count how many `..` segments are actually required to reach the import's
        // target from the current directory, and propose trimming the excess
        // (e.g. `../files/x.js` from within `files` should be `./x.js`). The target
        // of a relative specifier is determined lexically, so this needs no
        // filesystem access, and trimming excess `..` hops cannot change resolution.
        let trimmed = import_path.strip_prefix("./").unwrap_or(import_path);
        let count_import_parents = trimmed.split('/').filter(|part| *part == "..").count();
        if count_import_parents == 0 {
            return;
        }
        let target = lexical_join(current_dir, import_path);
        let count_expected_parents = relative_parent_count(current_dir, &target);
        if count_import_parents <= count_expected_parents {
            return;
        }
        let diff = count_import_parents - count_expected_parents;

        let parts: Vec<&str> = trimmed.split('/').collect();
        let mut proposed_parts: Vec<&str> = Vec::with_capacity(parts.len());
        proposed_parts.extend(&parts[..count_expected_parents]);
        if count_import_parents + diff < parts.len() {
            proposed_parts.extend(&parts[count_import_parents + diff..]);
        }
        let proposed = to_relative_path(&proposed_parts.join("/"));

        Self::report(import_path, &proposed, span, is_template, false, ctx);
    }

    fn report(
        import_path: &str,
        proposed_path: &str,
        span: Span,
        is_template: bool,
        dangerous: bool,
        ctx: &LintContext<'_>,
    ) {
        let proposed = proposed_path.to_string();
        let fix_kind = if dangerous { FixKind::DangerousFix } else { FixKind::SafeFix };
        ctx.diagnostic_with_fix_of_kind(
            no_useless_path_segments_diagnostic(import_path, proposed_path, span),
            fix_kind,
            |fixer| {
                if is_template {
                    // Replace the whole template literal with a plain string
                    fixer.replace(span, format!("\"{proposed}\""))
                } else {
                    // Replace the contents of the string literal, keeping the quotes
                    fixer.replace(span.shrink(1), proposed)
                }
            },
        );
    }
}

/// Normalize a relative path like `path.posix.normalize` + ensuring a `./` prefix:
/// collapses `.` segments, applies `..` segments, removes duplicate and trailing
/// slashes. `./foo/../bar//` -> `./bar`, `./../x` -> `../x`, `./` -> `.`.
fn normalize_relative(path: &str) -> String {
    let mut stack: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                if matches!(stack.last(), None | Some(&"..")) {
                    stack.push("..");
                } else {
                    stack.pop();
                }
            }
            _ => stack.push(segment),
        }
    }
    if stack.is_empty() {
        return ".".to_string();
    }
    let mut normalized = String::with_capacity(path.len() + 2);
    if stack[0] != ".." {
        normalized.push_str("./");
    }
    for (i, segment) in stack.iter().enumerate() {
        if i > 0 {
            normalized.push('/');
        }
        normalized.push_str(segment);
    }
    normalized
}

/// Is `path` already in the normal form produced by [`normalize_relative`]?
/// Equivalent to `normalize_relative(path) == path` without allocating, so the
/// dominant already-normalized import case builds no strings.
fn is_normalized(path: &str) -> bool {
    if path == "." {
        return true;
    }
    let mut segments = path.split('/');
    match segments.next() {
        // `./a/b`: at least one segment, all plain
        Some(".") => {
            let mut any = false;
            for segment in segments {
                if matches!(segment, "" | "." | "..") {
                    return false;
                }
                any = true;
            }
            any
        }
        // `../../a/b`: a leading run of `..`, then only plain segments
        Some("..") => {
            let mut in_parent_run = true;
            for segment in segments {
                match segment {
                    ".." if in_parent_run => {}
                    "" | "." | ".." => return false,
                    _ => in_parent_run = false,
                }
            }
            true
        }
        _ => false,
    }
}

/// Upstream `toRelativePath`: strip one trailing `/`, then ensure the path starts
/// with `.` (prefix `./` otherwise).
fn to_relative_path(path: &str) -> String {
    let stripped = path.strip_suffix('/').unwrap_or(path);
    if stripped == "."
        || stripped == ".."
        || stripped.starts_with("./")
        || stripped.starts_with("../")
    {
        stripped.to_string()
    } else {
        format!("./{stripped}")
    }
}

/// Does the path point at an `index` file: `.*/index` or `.*/index<ext>`?
fn has_index_suffix(path: &str, extensions: &[CompactStr]) -> bool {
    if path.ends_with("/index") {
        return true;
    }
    extensions.iter().any(|extension| {
        path.strip_suffix(extension.as_str()).is_some_and(|stem| stem.ends_with("/index"))
    })
}

/// Join a relative specifier onto a base directory, resolving `.` and `..`
/// segments lexically.
fn lexical_join(base_dir: &Path, specifier: &str) -> PathBuf {
    let mut out = base_dir.to_path_buf();
    for segment in specifier.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            segment => out.push(segment),
        }
    }
    out
}

/// Number of `..` segments in the relative path from `from_dir` to `to`
/// (i.e. how many components of `from_dir` are not shared with `to`).
fn relative_parent_count(from_dir: &Path, to: &Path) -> usize {
    let common =
        from_dir.components().zip(to.components()).take_while(|(from, to)| from == to).count();
    from_dir.components().count() - common
}

#[test]
fn is_normalized_matches_normalize_relative() {
    let cases = [
        ".",
        "..",
        "./",
        "../",
        ".//",
        "./a",
        "./a/",
        "./a/b",
        "././a",
        "./a/./b",
        "./a//b",
        "./a/../b",
        "./..",
        "./../a",
        "../..",
        "../../a",
        "../a/..",
        "../a/../b",
        "../a/./b",
        "../a//b",
        "../a/b/",
        ".foo",
        ".foo/bar",
        "...",
        ".../a",
        "./index",
        "../index.js",
        "./a/index",
        "..a",
        "..a/b",
        "./a/b/..",
        "./a/...",
        "../../",
        ".././a",
        "./a/..",
    ];
    for case in cases {
        assert_eq!(
            is_normalized(case),
            normalize_relative(case) == case,
            "is_normalized mismatch for {case:?}"
        );
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let commonjs = Some(json!([{ "commonjs": true }]));
    let commonjs_index = Some(json!([{ "commonjs": true, "noUselessIndex": true }]));
    let index = Some(json!([{ "noUselessIndex": true }]));
    // `settings.import.extensions` mirrors eslint-plugin-import's `import/extensions`
    let custom_extensions =
        Some(json!({ "settings": { "import": { "extensions": [".custom"] } } }));

    let pass = vec![
        // CommonJS modules are not checked without the `commonjs` option
        (r#"require("./../import/malformed.js")"#, None, None),
        (r#"import "./malformed.js""#, None, None),
        (r#"import "./test-module""#, None, None),
        (r#"import ".""#, None, None),
        (r#"import "..""#, None, None),
        (r#"import "fs""#, None, None),
        // `noUselessIndex` is false by default
        (r#"import "../index""#, None, None),
        (r#"import "../my-custom-index""#, index.clone(), None),
        // Normalized non-`index` paths are fine with `noUselessIndex`
        (r#"import "./bar.js""#, index.clone(), None),
        (r#"import "./bar""#, index.clone(), None),
        (r#"import "./malformed.js""#, index.clone(), None),
        (r#"import "./malformed""#, index.clone(), None),
        (r#"import "./importType""#, index.clone(), None),
        (r#"import(".")"#, None, None),
        (r#"import("..")"#, None, None),
        (r#"import("fs").then(function(fs) {})"#, None, None),
        // `.custom` is not a known extension by default, so this is not an index import
        (r#"import "../index.custom""#, index.clone(), None),
        // With restricted extensions, `.js` is no longer treated as an index import
        (r#"import "../index.js""#, index.clone(), custom_extensions.clone()),
    ];

    let fail = vec![
        // CommonJS
        (r#"require("./../import/malformed.js")"#, commonjs.clone(), None),
        ("require(`./../import/malformed.js`)", commonjs.clone(), None),
        (r#"require("./../import/malformed")"#, commonjs.clone(), None),
        (r#"require("../import/malformed.js")"#, commonjs.clone(), None),
        (r#"require("../import/malformed")"#, commonjs.clone(), None),
        (r#"require("./test-module/")"#, commonjs.clone(), None),
        (r#"require("./")"#, commonjs.clone(), None),
        (r#"require("../")"#, commonjs.clone(), None),
        (r#"require("./deep//a")"#, commonjs.clone(), None),
        // CommonJS with `noUselessIndex`
        (r#"require("./bar/index.js")"#, commonjs_index.clone(), None),
        (r#"require("./bar/index")"#, commonjs_index.clone(), None),
        (r#"require("./importPath/")"#, commonjs_index.clone(), None),
        (r#"require("./importPath/index.js")"#, commonjs_index.clone(), None),
        (r#"require("./importType/index")"#, commonjs_index.clone(), None),
        (r#"require("./index")"#, commonjs_index.clone(), None),
        (r#"require("../index")"#, commonjs_index.clone(), None),
        (r#"require("../index.js")"#, commonjs_index.clone(), None),
        // ES modules
        (r#"import "./../import/malformed.js""#, None, None),
        (r#"import "./../import/malformed""#, None, None),
        (r#"import "../import/malformed.js""#, None, None),
        (r#"import "../import/malformed""#, None, None),
        (r#"import "./test-module/""#, None, None),
        (r#"import "./bar/""#, None, None),
        (r#"import "./""#, None, None),
        (r#"import "../""#, None, None),
        (r#"import "./deep//a""#, None, None),
        // ES modules with `noUselessIndex`
        (r#"import "./bar/""#, index.clone(), None),
        (r#"import "./bar/index.js""#, index.clone(), None),
        (r#"import "./bar/index""#, index.clone(), None),
        (r#"import "./importPath/""#, index.clone(), None),
        (r#"import "./importPath/index.js""#, index.clone(), None),
        (r#"import "./importPath/index""#, index.clone(), None),
        (r#"import "./index""#, index.clone(), None),
        (r#"import "../index""#, index.clone(), None),
        (r#"import "../index.js""#, index.clone(), None),
        // With `settings.import.extensions`, custom extensions count as index imports
        (r#"import "../index.custom""#, index.clone(), custom_extensions),
        // Export-from declarations
        (r#"export { foo } from "./../import/malformed.js""#, None, None),
        (r#"export * from "./../import/malformed.js""#, None, None),
        // Dynamic imports
        (r#"import("./")"#, None, None),
        (r#"import("../")"#, None, None),
        (r#"import("./deep//a")"#, None, None),
    ];

    let fix = vec![
        (
            r#"require("./../import/malformed.js")"#,
            r#"require("../import/malformed.js")"#,
            commonjs.clone(),
        ),
        (
            "require(`./../import/malformed.js`)",
            r#"require("../import/malformed.js")"#,
            commonjs.clone(),
        ),
        (r#"require("../import/malformed.js")"#, r#"require("./malformed.js")"#, commonjs.clone()),
        (r#"require("./test-module/")"#, r#"require("./test-module")"#, commonjs.clone()),
        (r#"require("./")"#, r#"require(".")"#, commonjs.clone()),
        (r#"require("../")"#, r#"require("..")"#, commonjs.clone()),
        (r#"require("./deep//a")"#, r#"require("./deep/a")"#, commonjs),
        (r#"require("./bar/index.js")"#, r#"require("./bar")"#, commonjs_index.clone()),
        (r#"require("./importPath/index.js")"#, r#"require("./importPath")"#, commonjs_index),
        (r#"import "./../import/malformed.js""#, r#"import "../import/malformed.js""#, None),
        (r#"import "../import/malformed.js""#, r#"import "./malformed.js""#, None),
        (r#"import "./""#, r#"import ".""#, None),
        (r#"import "../""#, r#"import "..""#, None),
        (r#"import "./bar/""#, r#"import "./bar""#, None),
        (r#"import "./deep//a""#, r#"import "./deep/a""#, None),
        (r#"import "./bar/index""#, r#"import "./bar""#, index.clone()),
        (r#"import "./index""#, r#"import ".""#, index.clone()),
        (r#"import "../index.js""#, r#"import "..""#, index),
        (r#"import("./deep//a")"#, r#"import("./deep/a")"#, None),
    ];

    Tester::new(NoUselessPathSegments::NAME, NoUselessPathSegments::PLUGIN, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
