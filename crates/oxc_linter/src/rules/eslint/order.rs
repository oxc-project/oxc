use std::sync::LazyLock;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Clone, Deserialize)]
struct OrderConfig {
    groups: Option<Vec<CompactStr>>,
    #[serde(rename = "pathGroups")]
    path_groups: Option<Vec<PathGroup>>,
    #[serde(rename = "pathGroupsExcludedImportTypes")]
    path_groups_excluded_import_types: Option<Vec<CompactStr>>,
    #[serde(rename = "newlines-between")]
    newlines_between: Option<CompactStr>,
    named: Option<NamedOrder>,
    alphabetize: Option<Alphabetize>,
    #[serde(rename = "warnOnUnassignedImports")]
    warn_on_unassigned_imports: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct PathGroup {
    pattern: CompactStr,
    #[serde(rename = "patternOptions")]
    pattern_options: Option<PatternOptions>,
    group: String,
    position: Option<CompactStr>,
}

#[derive(Debug, Clone, Deserialize)]
struct PatternOptions {
    #[serde(rename = "nocomment")]
    no_comment: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct NamedOrder {
    enabled: Option<bool>,
    import: Option<bool>,
    export: Option<bool>,
    require: Option<bool>,
    #[serde(rename = "cjsExports")]
    cjs_exports: Option<bool>,
    types: Option<CompactStr>,
}

#[derive(Debug, Clone, Deserialize)]
struct Alphabetize {
    order: Option<CompactStr>,
    #[serde(rename = "orderImportKind")]
    order_import_kind: Option<CompactStr>,
    #[serde(rename = "caseInsensitive")]
    case_insensitive: Option<bool>,
}

#[derive(Debug, Default, Clone)]
pub struct Order {
    config: Option<OrderConfig>,
}

#[derive(Debug)]
struct ImportInfo {
    source: CompactStr,
    span: Span,
    group: CompactStr,
    rank: usize,
}

static BUILTIN_MODULES: LazyLock<rustc_hash::FxHashSet<&'static str>> = LazyLock::new(|| {
    let mut set = rustc_hash::FxHashSet::default();
    set.extend([
        "assert",
        "buffer",
        "child_process",
        "cluster",
        "crypto",
        "dgram",
        "dns",
        "domain",
        "events",
        "fs",
        "http",
        "https",
        "net",
        "os",
        "path",
        "punycode",
        "querystring",
        "readline",
        "stream",
        "string_decoder",
        "tls",
        "tty",
        "url",
        "util",
        "v8",
        "vm",
        "zlib",
    ]);
    set
});

declare_oxc_lint!(
    /// ### What it does
    /// Enforces a convention in module import order.
    ///
    /// ### Why is this bad?
    /// Having a consistent order in imports helps readability and maintainability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import _ from 'lodash';
    /// import path from 'path'; // `path` import should occur before import of `lodash`
    ///
    /// // -----
    ///
    /// var _ = require('lodash');
    /// var path = require('path'); // `path` import should occur before import of `lodash`
    ///
    /// // -----
    ///
    /// var path = require('path');
    /// import foo from './foo'; // `import` statements must be before `require` statement
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import path from 'path';
    /// import _ from 'lodash';
    ///
    /// // -----
    ///
    /// var path = require('path');
    /// var _ = require('lodash');
    ///
    /// // -----
    ///
    /// // Allowed as ̀`babel-register` is not assigned.
    /// require('babel-register');
    /// var path = require('path');
    ///
    /// // -----
    ///
    /// // Allowed as `import` must be before `require`
    /// import foo from './foo';
    /// var path = require('path');
    /// ```
    Order,
    style,
    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for Order {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self { config: serde_json::from_value(value).ok() }
    }
    fn run_once(&self, ctx: &LintContext) {
        if let Some(config) = &self.config {
            let mut imports = self.collect_imports(ctx);
            self.check_imports_order(ctx, &mut imports, config);
        }
    }
}

fn compare_sources(a: &str, b: &str, case_insensitive: bool) -> std::cmp::Ordering {
    if case_insensitive {
        let a_chars = a.chars().map(|c| c.to_ascii_lowercase());
        let b_chars = b.chars().map(|c| c.to_ascii_lowercase());
        a_chars.cmp(b_chars)
    } else {
        a.cmp(b)
    }
}

impl Order {
    fn collect_imports(&self, ctx: &LintContext) -> Vec<ImportInfo> {
        let mut imports = Vec::new();
        let module_record = ctx.module_record();

        // Collect import declarations
        for entry in &module_record.import_entries {
            let source = entry.module_request.name();
            let span = entry.module_request.span();

            imports.push(ImportInfo {
                source: CompactStr::new(source),
                span,
                group: CompactStr::new(self.get_import_group(source).as_str()),
                rank: 0,
            });
        }

        // Collect export from declarations
        for entry in &module_record.indirect_export_entries {
            if let Some(module_request) = &entry.module_request {
                let source = module_request.name();
                imports.push(ImportInfo {
                    source: CompactStr::new(source),
                    span: entry.span,
                    group: CompactStr::new(self.get_import_group(source).as_str()),
                    rank: 0,
                });
            }
        }

        imports
    }

    fn get_import_group(&self, source: &str) -> String {
        // Default groups: builtin, external, parent, sibling, index
        if source.starts_with('.') {
            if source == "." || source == ".." {
                "parent".into()
            } else if source.starts_with("./") {
                "sibling".into()
            } else {
                "parent".into()
            }
        } else if self.is_builtin_module(source) {
            "builtin".into()
        } else {
            "external".into()
        }
    }

    fn is_builtin_module(&self, source: &str) -> bool {
        BUILTIN_MODULES.contains(&source)
    }

    fn check_imports_order(
        &self,
        ctx: &LintContext,
        imports: &mut [ImportInfo],
        config: &OrderConfig,
    ) {
        // Assign ranks based on groups
        self.assign_ranks(imports, config);

        // Check alphabetical order if configured
        if let Some(alphabetize) = &config.alphabetize {
            self.check_alphabetical_order(ctx, imports, alphabetize);
        }

        // Check newlines between imports if configured
        if let Some(newlines_between) = &config.newlines_between {
            self.check_newlines_between(ctx, imports, newlines_between);
        }

        // Check for out of order imports
        self.check_order_violations(ctx, imports);
    }

    fn check_order_violations(&self, ctx: &LintContext, imports: &[ImportInfo]) {
        for i in 1..imports.len() {
            let prev = &imports[i - 1];
            let curr = &imports[i];

            if curr.rank < prev.rank {
                ctx.diagnostic(
                    OxcDiagnostic::warn(format!(
                        "Import from '{}' should occur before import from '{}'",
                        curr.source, prev.source
                    ))
                    .with_label(curr.span),
                );
            }
        }
    }

    // 3. Functions for ranking and group assignment
    fn assign_ranks(&self, imports: &mut [ImportInfo], config: &OrderConfig) {
        let group_ranks = self.get_group_ranks(config);

        for import in imports.iter_mut() {
            import.rank = self.calculate_rank(&import.group, &group_ranks);

            // Apply path group rankings if configured
            if let Some(path_groups) = &config.path_groups {
                if let Some(path_group_rank) = self.get_path_group_rank(&import.source, path_groups)
                {
                    import.rank = path_group_rank;
                }
            }
        }
    }

    fn get_group_ranks(&self, config: &OrderConfig) -> FxHashMap<CompactStr, usize> {
        let mut default_groups = FxHashMap::default();
        default_groups.insert(CompactStr::new("builtin"), 0);
        default_groups.insert(CompactStr::new("external"), 1);
        default_groups.insert(CompactStr::new("parent"), 2);
        default_groups.insert(CompactStr::new("sibling"), 3);
        default_groups.insert(CompactStr::new("index"), 4);

        if config.groups.is_none() {
            return default_groups;
        }

        let groups = config.groups.as_ref().unwrap();
        let mut ranks = FxHashMap::default();

        for (index, group) in groups.iter().enumerate() {
            ranks.insert(group.clone(), index);
        }

        ranks
    }

    fn calculate_rank(&self, group: &str, group_ranks: &FxHashMap<CompactStr, usize>) -> usize {
        *group_ranks.get(group).unwrap_or(&usize::MAX)
    }

    // 4. Functions for path group handling
    fn get_path_group_rank(&self, source: &str, path_groups: &[PathGroup]) -> Option<usize> {
        for (index, path_group) in path_groups.iter().enumerate() {
            if self.matches_pattern(source, &path_group.pattern) {
                let base_rank = index * 100; // Use multiplier to leave room for position adjustments

                // Adjust rank based on position if specified
                match path_group.position.as_deref() {
                    Some("before") => return Some(base_rank.saturating_sub(50)),
                    Some("after") => return Some(base_rank + 50),
                    _ => return Some(base_rank),
                }
            }
        }
        None
    }

    fn matches_pattern(&self, source: &str, pattern: &str) -> bool {
        // Simple pattern matching implementation
        // Could be enhanced with proper glob matching
        if pattern.contains('*') {
            let pattern = pattern.replace('*', ".*");
            regex::Regex::new(&pattern).map(|re| re.is_match(source)).unwrap_or(false)
        } else {
            source == pattern
        }
    }

    // 5. Functions for alphabetical ordering
    fn check_alphabetical_order(
        &self,
        ctx: &LintContext,
        imports: &[ImportInfo],
        alphabetize: &Alphabetize,
    ) {
        let case_insensitive = alphabetize.case_insensitive.unwrap_or(false);
        let order = alphabetize.order.as_deref().unwrap_or("ignore");

        if order == "ignore" {
            return;
        }

        for window in imports.windows(2) {
            let prev = &window[0];
            let curr = &window[1];

            // Only compare imports within the same group
            if prev.rank != curr.rank {
                continue;
            }

            let ordering = compare_sources(&prev.source, &curr.source, case_insensitive);

            let is_wrong_order = match order {
                "asc" => ordering == std::cmp::Ordering::Greater,
                "desc" => ordering == std::cmp::Ordering::Less,
                _ => false,
            };

            if is_wrong_order {
                ctx.diagnostic(
                    OxcDiagnostic::warn(format!(
                        "Imports must be sorted in {} order. '{}' should be before '{}'.",
                        order, curr.source, prev.source
                    ))
                    .with_label(curr.span),
                );
            }
        }
    }

    // 6. Functions for newlines checking
    fn check_newlines_between(
        &self,
        ctx: &LintContext,
        imports: &[ImportInfo],
        newlines_setting: &str,
    ) {
        if newlines_setting == "ignore" {
            return;
        }

        let source_code = ctx.source_text();

        for window in imports.windows(2) {
            let prev = &window[0];
            let curr = &window[1];

            let lines_between = self.count_newlines_between(
                source_code,
                prev.span.end.try_into().unwrap(),
                curr.span.start.try_into().unwrap(),
            );

            // Only check for newlines when transitioning between different groups
            let is_different_group = self.is_different_group(prev, curr);

            match newlines_setting {
                "always" => {
                    if is_different_group && lines_between == 0 {
                        ctx.diagnostic(
                            OxcDiagnostic::warn(
                                "There should be at least one empty line between import groups",
                            )
                            .with_label(curr.span),
                        );
                    }
                }
                "never" => {
                    if lines_between > 0 {
                        ctx.diagnostic(
                            OxcDiagnostic::warn("There should be no empty lines between imports")
                                .with_label(curr.span),
                        );
                    }
                }
                "always-and-inside-groups" => {
                    if lines_between == 0 {
                        ctx.diagnostic(
                            OxcDiagnostic::warn(
                                "There should be at least one empty line between imports",
                            )
                            .with_label(curr.span),
                        );
                    }
                }
                _ => {}
            }
        }
    }

    fn is_different_group(&self, prev: &ImportInfo, curr: &ImportInfo) -> bool {
        // Compare the base group names rather than ranks
        // This ensures we only require newlines between actual different groups
        prev.group != curr.group
    }

    fn count_newlines_between(&self, source: &str, start: usize, end: usize) -> usize {
        source[start..end].chars().filter(|&c| c == '\n').count().saturating_sub(1)
        // Subtract 1 because we don't count the line with the import
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Basic sorting
        (
            r#"
            import fs from 'fs';
            import path from 'path';

            import _ from 'lodash';
            import chalk from 'chalk';

            import foo from '../foo';

            import bar from './bar';
            "#,
            Some(serde_json::json!({
                "groups": ["builtin", "external", "parent", "sibling", "index"],
                "newlines-between": "always"
            })),
        ),
        // Alphabetical order
        (
            r#"
            import a from 'a';
            import b from 'b';
            import c from 'c';
            "#,
            Some(serde_json::json!({
                "alphabetize": {
                    "order": "asc",
                    "caseInsensitive": true
                }
            })),
        ),
        // Mixed groups with correct newlines
        (
            r#"
            import path from 'path';
            import fs from 'fs';

            import _ from 'lodash';

            import foo from '../foo';
            import bar from './bar';
            "#,
            Some(serde_json::json!({
                "groups": ["builtin", "external", ["parent", "sibling"]],
                "newlines-between": "always"
            })),
        ),
    ];

    let fail = vec![
        // Wrong order
        (
            r#"
            import _ from 'lodash';
            import fs from 'fs';
            "#,
            Some(serde_json::json!({
                "groups": ["builtin", "external"]
            })),
        ),
        // Missing newline between groups
        (
            r#"
            import fs from 'fs';
            import _ from 'lodash';  // Should have newline before this
            "#,
            Some(serde_json::json!({
                "groups": ["builtin", "external"],
                "newlines-between": "always"
            })),
        ),
        // Wrong alphabetical order
        (
            r#"
            import b from 'b';
            import a from 'a';
            "#,
            Some(serde_json::json!({
                "alphabetize": {
                    "order": "asc"
                }
            })),
        ),
    ];

    Tester::new(Order::NAME, Order::CATEGORY, pass, fail).test_and_snapshot();
}
