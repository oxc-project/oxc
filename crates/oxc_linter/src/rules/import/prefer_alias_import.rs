use std::path::{Path, PathBuf};

use cow_utils::CowUtils;
use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn prefer_alias_diagnostic(span: Span, import_path: &str, suggested_alias: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer using path alias instead of relative import")
        .with_help(format!("Replace '{}' with '{}'", import_path, suggested_alias))
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AliasConfig {
    /// Simple string path mapping: "@/*" -> "./src/*"
    Single(String),
    /// Multiple path mappings (TypeScript style): "@/*" -> ["./src/*", "./lib/*"]
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PreferAliasImportConfig {
    /// Path aliases mapping.
    ///
    /// Format matches TypeScript's paths configuration:
    /// ```json
    /// {
    ///   "@/*": "./src/*",
    ///   "@components/*": "./src/components/*",
    ///   "@utils/*": ["./src/utils/*", "./lib/utils/*"]
    /// }
    /// ```
    #[serde(default)]
    pub aliases: std::collections::HashMap<String, AliasConfig>,

    /// Minimum depth (number of `../` segments) before suggesting an alias.
    ///
    /// Examples:
    /// - `1` (default): `../foo` triggers the rule
    /// - `2`: only `../../foo` and deeper trigger the rule
    #[serde(default = "default_min_depth")]
    pub min_depth: usize,
}

fn default_min_depth() -> usize {
    1
}

impl Default for PreferAliasImportConfig {
    fn default() -> Self {
        Self { aliases: std::collections::HashMap::default(), min_depth: default_min_depth() }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PreferAliasImport(Box<PreferAliasImportConfig>);

impl std::ops::Deref for PreferAliasImport {
    type Target = PreferAliasImportConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of path aliases instead of relative imports.
    ///
    /// ### Why is this bad?
    ///
    /// Using path aliases makes imports cleaner and more maintainable:
    /// - No more `../../../` hell
    /// - Imports remain valid when moving files around
    /// - More readable and consistent codebase
    /// - Easier to refactor and reorganize code
    ///
    /// ### Configuration
    ///
    /// ```json
    /// {
    ///   "aliases": {
    ///     "@/*": "./src/*",
    ///     "@components/*": "./src/components/*"
    ///   },
    ///   "min_depth": 1
    /// }
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import { Button } from '../../../components/Button';
    /// import { helper } from '../../utils/helper';
    /// const config = require('../config');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// import { Button } from '@/components/Button';
    /// import { helper } from '@/utils/helper';
    /// const config = require('@/config');
    /// ```
    PreferAliasImport,
    import,
    style,
    config = PreferAliasImportConfig,
);

impl Rule for PreferAliasImport {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Extract (path, span) from different import/export types
        let (import_path, span) = match node.kind() {
            AstKind::ImportDeclaration(decl) => (decl.source.value.as_str(), decl.source.span),
            AstKind::ExportNamedDeclaration(decl) => {
                let Some(source) = &decl.source else { return };
                (source.value.as_str(), source.span)
            }
            AstKind::ExportAllDeclaration(decl) => (decl.source.value.as_str(), decl.source.span),
            AstKind::ImportExpression(expr) => {
                let Expression::StringLiteral(lit) = &expr.source else { return };
                (lit.value.as_str(), lit.span)
            }
            AstKind::CallExpression(call) => {
                let Expression::Identifier(ident) = &call.callee else { return };
                if ident.name != "require" || call.arguments.len() != 1 {
                    return;
                }
                let Argument::StringLiteral(lit) = &call.arguments[0] else { return };
                (lit.value.as_str(), lit.span)
            }
            _ => return,
        };

        if let Some(alias) = self.find_matching_alias(import_path, ctx) {
            ctx.diagnostic(prefer_alias_diagnostic(span, import_path, &alias));
        }
    }
}

impl PreferAliasImport {
    /// Checks if the import path should use an alias instead of a relative path
    /// Returns the suggested alias path if one matches
    fn find_matching_alias(&self, import_path: &str, ctx: &LintContext) -> Option<String> {
        // Only check relative imports (starting with ./ or ../)
        if !import_path.starts_with("./") && !import_path.starts_with("../") {
            return None;
        }

        // Check if import meets minimum depth requirement.
        // Note: min_depth only applies to parent traversals (../).
        // Imports starting with "./" (same directory) are always skipped
        // since they have 0 parent traversals and min_depth defaults to 1.
        let parent_count = import_path.matches("../").count();
        if parent_count < self.min_depth {
            return None;
        }

        // Get the current file's path
        let current_file = ctx.file_path();
        let current_dir = current_file.parent()?;

        // Resolve the absolute path of the import
        let resolved_import_path = self.resolve_import_path(current_dir, import_path)?;

        // Try to find a matching alias
        self.find_best_alias(&resolved_import_path)
    }

    /// Resolves a relative import path to an absolute path
    fn resolve_import_path(&self, base_dir: &Path, import_path: &str) -> Option<PathBuf> {
        let mut path = base_dir.to_path_buf();

        // Process each segment of the import path
        for segment in import_path.split('/') {
            match segment {
                "." | "" => continue,
                ".." => {
                    path.pop();
                }
                _ => {
                    path.push(segment);
                }
            }
        }

        Some(path)
    }

    /// Find the best matching alias for the given absolute path
    /// Prioritizes more specific aliases (longer matching paths)
    fn find_best_alias(&self, target_path: &Path) -> Option<String> {
        let mut best_match: Option<(String, usize)> = None;

        for (alias_pattern, alias_config) in &self.aliases {
            // Convert AliasConfig to Vec<String>
            let alias_paths = match alias_config {
                AliasConfig::Single(path) => vec![path.clone()],
                AliasConfig::Multiple(paths) => paths.clone(),
            };

            // Process each configured path for this alias
            for alias_path in alias_paths {
                if let Some((alias_replacement, match_length)) =
                    self.try_match_alias(target_path, alias_pattern, &alias_path)
                {
                    // Keep the most specific match (longest matching base path)
                    if best_match.as_ref().map_or(true, |(_, len)| match_length > *len) {
                        best_match = Some((alias_replacement, match_length));
                    }
                }
            }
        }

        best_match.map(|(path, _)| path)
    }

    /// Try to match a target path against an alias pattern
    /// Returns (suggested_alias, match_length) if successful
    fn try_match_alias(
        &self,
        target_path: &Path,
        alias_pattern: &str,
        alias_path: &str,
    ) -> Option<(String, usize)> {
        // Strip the wildcard from patterns
        // e.g., "@app/*" -> "@app", "./src/app/*" -> "./src/app"
        let alias_base = alias_pattern.trim_end_matches("/*");
        let path_base = alias_path.trim_start_matches("./").trim_end_matches("/*");

        // Convert target path to string for matching
        // Normalize Windows backslashes to forward slashes for consistent matching
        let target_lossy = target_path.to_string_lossy();
        let target_str = target_lossy.cow_replace('\\', "/");

        // Try to find the path_base within the target path
        // We need to match path components, not just string substrings
        // Look for "/path_base/" or path starting with "path_base/"
        let search_pattern = format!("/{path_base}/");
        let start_pattern = format!("{path_base}/");

        let relative_part = if let Some(idx) = target_str.find(&search_pattern) {
            // Found in the middle of the path
            let after_base = idx + search_pattern.len();
            &target_str[after_base..]
        } else if target_str.starts_with(&start_pattern) {
            // Path starts with the base
            &target_str[start_pattern.len()..]
        } else if target_str.as_ref() == path_base {
            // Exact match
            ""
        } else {
            return None;
        };

        // Construct the suggested alias path
        let suggested = if relative_part.is_empty() {
            alias_base.to_string()
        } else {
            format!("{}/{}", alias_base, relative_part)
        };

        // Return the suggestion with the length of the matching base
        // (used to prioritize more specific aliases)
        Some((suggested, path_base.len()))
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        // Already using aliases
        (
            r#"import { Button } from '@/components/Button'"#,
            Some(json!([{ "aliases": { "@/*": "./src/*" } }])),
        ),
        (
            r#"import helper from '@app/utils/helper'"#,
            Some(json!([{ "aliases": { "@app/*": "./src/app/*" } }])),
        ),
        (
            r#"const config = require('@/config')"#,
            Some(json!([{ "aliases": { "@/*": "./src/*" } }])),
        ),
        // External packages (should be ignored)
        (r#"import React from 'react'"#, Some(json!([{ "aliases": { "@/*": "./src/*" } }]))),
        (r#"import lodash from 'lodash'"#, Some(json!([{ "aliases": { "@/*": "./src/*" } }]))),
        (
            r#"import { Button } from '@mui/material'"#,
            Some(json!([{ "aliases": { "@/*": "./src/*" } }])),
        ),
        // Single-level relative imports with min_depth: 2
        (
            r#"import { foo } from '../sibling'"#,
            Some(json!([{ "aliases": { "@/*": "./src/*" }, "min_depth": 2 }])),
        ),
        (
            r#"import { bar } from './local'"#,
            Some(json!([{ "aliases": { "@/*": "./src/*" }, "min_depth": 2 }])),
        ),
        // No aliases configured
        (r#"import { foo } from '../bar'"#, None),
        // Multiple aliases - using the correct one
        (
            r#"import { Button } from '@components/Button'"#,
            Some(json!([{
                "aliases": {
                    "@components/*": "./src/components/*",
                    "@utils/*": "./src/utils/*",
                    "@/*": "./src/*"
                }
            }])),
        ),
    ];

    let fail = vec![
        // Basic relative imports that could use aliases
        (
            r#"import { Button } from '../../../components/Button'"#,
            Some(json!([{ "aliases": { "@/*": "./src/*" } }])),
        ),
        (
            r#"import helper from '../../utils/helper'"#,
            Some(json!([{ "aliases": { "@/*": "./src/*" } }])),
        ),
        (
            r#"const config = require('../../../config')"#,
            Some(json!([{ "aliases": { "@/*": "./src/*" } }])),
        ),
        // Exports
        (
            r#"export { Button } from '../../components/Button'"#,
            Some(json!([{ "aliases": { "@/*": "./src/*" } }])),
        ),
        (r#"export * from '../../../utils'"#, Some(json!([{ "aliases": { "@/*": "./src/*" } }]))),
        // Dynamic imports
        (
            r#"import('../../../dynamic/module')"#,
            Some(json!([{ "aliases": { "@/*": "./src/*" } }])),
        ),
        // Multiple specific aliases - should use the most specific one
        (
            r#"import { Button } from '../../components/Button'"#,
            Some(json!([{
                "aliases": {
                    "@components/*": "./src/components/*",
                    "@utils/*": "./src/utils/*",
                    "@/*": "./src/*"
                }
            }])),
        ),
        (
            r#"import { helper } from '../../utils/helper'"#,
            Some(json!([{
                "aliases": {
                    "@components/*": "./src/components/*",
                    "@utils/*": "./src/utils/*",
                    "@/*": "./src/*"
                }
            }])),
        ),
        // min_depth configuration
        (
            r#"import { foo } from '../bar'"#,
            Some(json!([{
                "aliases": { "@/*": "./src/*" },
                "min_depth": 1
            }])),
        ),
        (
            r#"import { foo } from '../../bar'"#,
            Some(json!([{
                "aliases": { "@/*": "./src/*" },
                "min_depth": 2
            }])),
        ),
        // App-specific aliases
        (
            r#"import { UserService } from '../../../app/services/UserService'"#,
            Some(json!([{
                "aliases": {
                    "@app/*": "./src/app/*"
                }
            }])),
        ),
        // Multiple path mappings (array style)
        (
            r#"import { foo } from '../../lib/utils/foo'"#,
            Some(json!([{
                "aliases": {
                    "@utils/*": ["./src/utils/*", "./lib/utils/*"]
                }
            }])),
        ),
    ];

    Tester::new(PreferAliasImport::NAME, PreferAliasImport::PLUGIN, pass, fail)
        .change_rule_path("src/pages/deep/nested/component.tsx")
        .with_import_plugin(true)
        .test_and_snapshot();
}