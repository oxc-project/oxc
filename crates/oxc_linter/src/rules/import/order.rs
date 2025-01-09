use cow_utils::CowUtils;
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
    #[serde(rename = "newlines-between")]
    newlines_between: Option<CompactStr>,
    alphabetize: Option<Alphabetize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
enum PredefinedGroup {
    Builtin,
    External,
    Internal,
    Parent,
    Sibling,
    Index,
    Object,
}

#[derive(Debug, Clone, Deserialize)]
struct PathGroup {
    pattern: CompactStr,
    #[serde(rename = "group")]
    group: PredefinedGroup,
    position: Option<CompactStr>,
}

#[derive(Debug, Clone, Deserialize)]
struct Alphabetize {
    order: Option<CompactStr>,
    #[serde(rename = "caseInsensitive")]
    case_insensitive: Option<bool>,
}

#[derive(Debug, Default, Clone)]
pub struct Order {
    config: Option<Box<OrderConfig>>,
}

#[derive(Debug)]
struct ImportInfo {
    source: CompactStr,
    span: Span,
    group: CompactStr,
    rank: usize,
}

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
    import,
    nursery
);

impl Rule for Order {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self { config: serde_json::from_value(value).ok().map(Box::new) }
    }
    fn run_once(&self, ctx: &LintContext) {
        if let Some(config) = &self.config {
            let mut imports = collect_imports(ctx);
            check_imports_order(ctx, &mut imports, config);
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

fn collect_imports(ctx: &LintContext) -> Vec<ImportInfo> {
    let mut imports = Vec::new();
    let module_record = ctx.module_record();

    for entry in &module_record.import_entries {
        let source = entry.module_request.name();
        let span = entry.module_request.span();

        imports.push(ImportInfo {
            source: CompactStr::new(source),
            span,
            group: CompactStr::new(get_import_group(source).as_str()),
            rank: 0,
        });
    }

    for entry in &module_record.indirect_export_entries {
        if let Some(module_request) = &entry.module_request {
            let source = module_request.name();
            imports.push(ImportInfo {
                source: CompactStr::new(source),
                span: entry.span,
                group: CompactStr::new(get_import_group(source).as_str()),
                rank: 0,
            });
        }
    }

    imports
}

fn check_imports_order(ctx: &LintContext, imports: &mut [ImportInfo], config: &OrderConfig) {
    assign_ranks(imports, config);
    check_all_rules(ctx, imports, config);
}

fn check_all_rules(ctx: &LintContext, imports: &[ImportInfo], config: &OrderConfig) {
    if imports.len() <= 1 {
        return;
    }

    let source_code = ctx.source_text();
    let alphabetize = &config.alphabetize;
    let newlines_setting = config.newlines_between.as_deref();

    // Get alphabetization settings if enabled
    let (check_alpha, alpha_case_insensitive, alpha_order) = if let Some(alpha) = alphabetize {
        (
            alpha.order.as_deref() != Some("ignore"),
            alpha.case_insensitive.unwrap_or(false),
            alpha.order.as_deref().unwrap_or("ignore"),
        )
    } else {
        (false, false, "ignore")
    };

    // Single pass through imports checking all rules
    for i in 1..imports.len() {
        let prev = &imports[i - 1];
        let curr = &imports[i];

        // Check order violations
        if curr.rank < prev.rank {
            let message = if curr.rank % 100 != 0 {
                format!(
                    "Import from '{}' should occur {} import from '{}'",
                    curr.source,
                    if curr.rank % 100 == 50 { "after" } else { "before" },
                    prev.source
                )
            } else {
                format!(
                    "Import from '{}' should occur before import from '{}'",
                    curr.source, prev.source
                )
            };
            ctx.diagnostic(OxcDiagnostic::warn(message).with_label(curr.span));
        }

        // Check alphabetical order within same group
        if check_alpha && prev.rank == curr.rank {
            let ordering = compare_sources(&prev.source, &curr.source, alpha_case_insensitive);
            let is_wrong_order = match alpha_order {
                "asc" => ordering == std::cmp::Ordering::Greater,
                "desc" => ordering == std::cmp::Ordering::Less,
                _ => false,
            };

            if is_wrong_order {
                ctx.diagnostic(
                    OxcDiagnostic::warn(format!(
                        "Imports must be sorted in {} order. '{}' should be before '{}'.",
                        alpha_order, curr.source, prev.source
                    ))
                    .with_label(curr.span),
                );
            }
        }

        // Check newlines between imports
        if let Some(newlines_setting) = newlines_setting {
            if newlines_setting != "ignore" {
                let lines_between = count_newlines_between(
                    source_code,
                    prev.span.end.try_into().unwrap(),
                    curr.span.start.try_into().unwrap(),
                );
                let is_different_group = prev.group != curr.group;

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
                                OxcDiagnostic::warn(
                                    "There should be no empty lines between imports",
                                )
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
    }
}

fn get_import_group(source: &str) -> String {
    if source.starts_with('.') {
        if source == "." || source == ".." {
            "parent".into()
        } else if source.starts_with("./") {
            "sibling".into()
        } else {
            "parent".into()
        }
    } else if is_builtin_module(source) {
        "builtin".into()
    } else {
        "external".into()
    }
}

fn is_builtin_module(source: &str) -> bool {
    let mut builtin_modules = rustc_hash::FxHashSet::default();
    builtin_modules.extend([
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

    builtin_modules.contains(&source)
}

fn assign_ranks(imports: &mut [ImportInfo], config: &OrderConfig) {
    let group_ranks = get_group_ranks(config);

    for import in imports.iter_mut() {
        import.rank = calculate_rank(&import.group, &group_ranks);
        if let Some(path_groups) = &config.path_groups {
            if let Some(path_group_rank) = get_path_group_rank(&import.source, path_groups) {
                import.rank = path_group_rank;
            }
        }
    }
}

fn get_group_ranks(config: &OrderConfig) -> FxHashMap<CompactStr, usize> {
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

fn calculate_rank(group: &str, group_ranks: &FxHashMap<CompactStr, usize>) -> usize {
    match group {
        "builtin" => 0,
        "external" => 100,
        "internal" => 200,
        "parent" => 300,
        "sibling" => 400,
        "index" => 500,
        _ => *group_ranks.get(group).unwrap_or(&(usize::MAX / 100)) * 100,
    }
}

fn get_path_group_rank(source: &str, path_groups: &[PathGroup]) -> Option<usize> {
    for path_group in path_groups {
        if matches_pattern(source, &path_group.pattern) {
            let target_group_rank = get_predefined_group_rank(&path_group.group);
            let base_rank = target_group_rank * 100; // Multiply by 100 to leave space for positioning

            match path_group.position.as_deref() {
                Some("before") => return Some(base_rank - 10),
                Some("after") => return Some(base_rank + 110), // Add more than 100 to ensure it's after the next group
                _ => return Some(base_rank),
            }
        }
    }
    None
}

fn get_predefined_group_rank(group: &PredefinedGroup) -> usize {
    match group {
        PredefinedGroup::Builtin => 0,
        PredefinedGroup::External => 1,
        PredefinedGroup::Internal => 2,
        PredefinedGroup::Parent => 3,
        PredefinedGroup::Sibling => 4,
        PredefinedGroup::Index => 5,
        PredefinedGroup::Object => 6,
    }
}

fn matches_pattern(source: &str, pattern: &str) -> bool {
    // Handle regular glob patterns
    if pattern.contains('*') {
        let escaped = pattern.cow_replace('.', "\\.");
        let with_temp_stars = escaped.cow_replace("**", "__DOUBLE_STAR__");
        let with_single_stars = with_temp_stars.cow_replace('*', "[^/]*");
        let regex_pattern = with_single_stars.cow_replace("__DOUBLE_STAR__", ".*");
        return regex::Regex::new(&format!("^{regex_pattern}$"))
            .map(|re| re.is_match(source))
            .unwrap_or(false);
    }

    // Exact match
    source == pattern
}

fn count_newlines_between(source: &str, start: usize, end: usize) -> usize {
    source[start..end].chars().filter(|&c| c == '\n').count().saturating_sub(1)
    // Subtract 1 because we don't count the line with the import
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Basic sorting
        (
            r"
            import fs from 'fs';
            import path from 'path';

            import _ from 'lodash';
            import chalk from 'chalk';

            import foo from '../foo';

            import bar from './bar';
            ",
            Some(serde_json::json!({
                "groups": ["builtin", "external", "parent", "sibling", "index"],
                "newlines-between": "always"
            })),
        ),
        // Alphabetical order
        (
            r"
            import a from 'a';
            import b from 'b';
            import c from 'c';
            ",
            Some(serde_json::json!({
                "alphabetize": {
                    "order": "asc",
                    "caseInsensitive": true
                }
            })),
        ),
        // Mixed groups with correct newlines
        (
            r"
            import path from 'path';
            import fs from 'fs';

            import _ from 'lodash';

            import foo from '../foo';
            import bar from './bar';
            ",
            Some(serde_json::json!({
                "groups": ["builtin", "external", ["parent", "sibling"]],
                "newlines-between": "always"
            })),
        ),
        // Test with pathGroups
        (
            r"
            import fs from 'fs';
            import _ from 'lodash';
            import MyComponent from '~/components/MyComponent';
            import utils from './utils';
            ",
            Some(serde_json::json!({
                "groups": ["builtin", "external", "internal", "parent", "sibling", "index"],
                "pathGroups": [{
                    "pattern": "~/components/**",
                    "group": "internal",
                    "position": "after"
                }]
            })),
        ),
    ];

    let fail = vec![
        // Wrong order
        (
            r"
            import _ from 'lodash';
            import fs from 'fs';
            ",
            Some(serde_json::json!({
                "groups": ["builtin", "external"]
            })),
        ),
        // Missing newline between groups
        (
            r"
            import fs from 'fs';
            import _ from 'lodash';  // Should have newline before this
            ",
            Some(serde_json::json!({
                "groups": ["builtin", "external"],
                "newlines-between": "always"
            })),
        ),
        // Wrong alphabetical order
        (
            r"
            import b from 'b';
            import a from 'a';
            ",
            Some(serde_json::json!({
                "alphabetize": {
                    "order": "asc"
                }
            })),
        ),
        (
            r"
            import MyComponent from '~/components/MyComponent';
            import _ from 'lodash';
            ",
            Some(serde_json::json!({
                "groups": ["builtin", "external", "internal"],
                "pathGroups": [{
                    "pattern": "~/components/**",
                    "group": "internal",
                    "position": "after"
                }]
            })),
        ),
    ];

    Tester::new(Order::NAME, Order::PLUGIN, pass, fail).test_and_snapshot();
}

#[test]
fn test_matches_pattern() {
    // Root-relative paths
    assert!(matches_pattern("~/components/Button", "~/components/**"));
    assert!(matches_pattern("~/components/forms/Input", "~/components/**"));
    assert!(!matches_pattern("other/Button", "~/components/**"));

    // Regular glob patterns
    assert!(matches_pattern("@org/utils", "@org/*"));
    assert!(matches_pattern("@org/deep/nested/util", "@org/**"));
    assert!(!matches_pattern("@org/deep/util", "@org/*"));

    // Exact matches
    assert!(matches_pattern("exact-match", "exact-match"));
    assert!(!matches_pattern("not-exact", "exact-match"));
}
