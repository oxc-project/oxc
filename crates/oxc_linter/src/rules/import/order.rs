use cow_utils::CowUtils;
use lazy_regex::Regex;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Clone, Deserialize)]
struct OrderConfig {
    groups: Option<Vec<GroupValue>>,
    #[serde(rename = "pathGroups")]
    path_groups: Option<Vec<PathGroup>>,
    #[serde(rename = "newlines-between")]
    alphabetize: Option<Alphabetize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum GroupValue {
    Single(CompactStr),
    Multiple(Vec<CompactStr>),
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum PredefinedGroup {
    Builtin,
    External,
    Internal,
    Parent,
    Sibling,
    Index,
    Object,
    Type,
    Unknown,
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

/// A minimal struct to store info about an import
#[derive(Debug, Clone)]
struct ImportInfo {
    /// The group this import belongs to
    group: PredefinedGroup,
    /// the import's position in the file as encountered
    original_index: usize,
    /// for generating a diagnostic
    span: Span,
    /// the import source, e.g. "fs" or "./bar"
    specifier: String,
    /// Path group this import matches, if any
    path_group_index: Option<usize>,
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
        // The configuration in tests is wrapped in an array
        let config = if value.is_array() {
            // Extract the first element from the array
            value
                .as_array()
                .and_then(|arr| arr.first().cloned())
                .and_then(|inner_value| serde_json::from_value(inner_value).ok())
                .map(Box::new)
        } else {
            // Direct configuration object
            serde_json::from_value(value).ok().map(Box::new)
        };

        Self { config }
    }
    fn run_once(&self, ctx: &LintContext<'_>) {
        // Gather all imports from the module record
        let module_record = ctx.module_record();
        let import_entries = &module_record.import_entries;

        // Early return if no imports to check
        if import_entries.is_empty() {
            return;
        }

        // Extract config or use defaults
        let default_groups = vec![
            GroupValue::Single("builtin".into()),
            GroupValue::Single("external".into()),
            GroupValue::Single("parent".into()),
            GroupValue::Single("sibling".into()),
            GroupValue::Single("index".into()),
        ];

        let groups = match &self.config {
            Some(config) => config.groups.as_ref().unwrap_or(&default_groups),
            None => &default_groups,
        };

        let alphabetize = self.config.as_ref().and_then(|c| c.alphabetize.as_ref());

        let path_groups = self.config.as_ref().and_then(|c| c.path_groups.as_ref());

        // Convert each import entry into our ImportInfo struct
        let imports: Vec<ImportInfo> = import_entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let specifier = entry.module_request.name();
                let group = classify_import_source(specifier);

                // Check if this import matches any path group
                let path_group_index = path_groups.and_then(|groups| {
                    groups.iter().position(|pg| matches_pattern(specifier, &pg.pattern))
                });

                ImportInfo {
                    group,
                    original_index: idx,
                    span: entry.statement_span,
                    specifier: specifier.to_owned(),
                    path_group_index,
                }
            })
            .collect();

        // Make a sorted clone to determine the ideal order
        let mut sorted_imports = imports.clone();
        sort_imports(&mut sorted_imports, groups, path_groups, alphabetize);

        // Create a mapping from original import to its expected position
        let mut import_positions: FxHashMap<usize, usize> = FxHashMap::default();
        for (expected_pos, import) in sorted_imports.iter().enumerate() {
            import_positions.insert(import.original_index, expected_pos);
        }

        // Check if the original order matches the sorted order
        for (original, sorted) in imports.iter().zip(&sorted_imports) {
            if original.original_index != sorted.original_index {
                ctx.diagnostic(
                    OxcDiagnostic::error(format!(
                        "Import from '{}' should occur before import from '{}'",
                        sorted.specifier, original.specifier
                    ))
                    .with_label(original.span),
                );

                // Only report the first disorder
                break;
            }
        }
    }
}

/// Sort the imports according to the configuration
fn sort_imports(
    imports: &mut [ImportInfo],
    groups: &[GroupValue],
    path_groups: Option<&Vec<PathGroup>>,
    alphabetize: Option<&Alphabetize>,
) {
    imports.sort_by(|a, b| {
        // 1. First compare by group rank according to configured groups
        let a_rank = get_group_rank(&a.group, a.path_group_index, groups, path_groups);
        let b_rank = get_group_rank(&b.group, b.path_group_index, groups, path_groups);

        let group_comparison = a_rank.cmp(&b_rank);
        if group_comparison != std::cmp::Ordering::Equal {
            return group_comparison;
        }

        // 2. If alphabetize is enabled, sort by specifier
        if let Some(alpha) = alphabetize {
            if let Some(order) = &alpha.order {
                if order == "asc" || order == "desc" {
                    let case_insensitive = alpha.case_insensitive.unwrap_or(false);

                    let a_spec = if case_insensitive {
                        a.specifier.cow_to_lowercase().to_string()
                    } else {
                        a.specifier.clone()
                    };

                    let b_spec = if case_insensitive {
                        b.specifier.cow_to_lowercase().to_string()
                    } else {
                        b.specifier.clone()
                    };

                    let comparison = a_spec.cmp(&b_spec);
                    if order == "desc" {
                        return comparison.reverse();
                    }
                    return comparison;
                }
            }
        }

        // 3. Preserve original order if no other sorting criteria apply
        a.original_index.cmp(&b.original_index)
    });
}

/// Get the rank of a group based on the specified order in configuration
fn get_group_rank(
    group: &PredefinedGroup,
    path_group_index: Option<usize>,
    groups: &[GroupValue],
    path_groups: Option<&Vec<PathGroup>>,
) -> usize {
    // If this import matches a path group, use its rank
    if let Some(pg_idx) = path_group_index {
        if let Some(path_groups) = path_groups {
            if let Some(path_group) = path_groups.get(pg_idx) {
                // Find the rank of the path group's target group
                for (i, group_value) in groups.iter().enumerate() {
                    match group_value {
                        GroupValue::Single(g) => {
                            if group_from_str(g) == Some(path_group.group.clone()) {
                                let position_modifier = match path_group.position.as_deref() {
                                    Some("after") => 1,
                                    _ => 0,
                                };
                                return i * 2 + position_modifier; // * 2 to make room for before/after
                            }
                        }
                        GroupValue::Multiple(gs) => {
                            for g in gs {
                                if group_from_str(g) == Some(path_group.group.clone()) {
                                    let position_modifier = match path_group.position.as_deref() {
                                        Some("after") => 1,
                                        _ => 0,
                                    };
                                    return i * 2 + position_modifier;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Store whether the group is found in the configuration
    let found = false;

    // Otherwise look for the group in the configuration
    for (i, group_value) in groups.iter().enumerate() {
        match group_value {
            GroupValue::Single(g) => {
                if group_from_str(g) == Some(group.clone()) {
                    return i * 2; // * 2 to accommodate possible path group positions
                }
            }
            GroupValue::Multiple(gs) => {
                for g in gs {
                    if group_from_str(g) == Some(group.clone()) {
                        return i * 2;
                    }
                }
            }
        }
    }

    // If group is not in configured groups, put it at the end
    // But ensure consistent ordering among non-configured groups
    if found {
        // Should never reach here if found is true
        usize::MAX
    } else {
        // Return a consistent rank based on the predefined order
        // This ensures stable sorting among non-configured groups
        match group {
            PredefinedGroup::Builtin => groups.len() * 2,
            PredefinedGroup::External => groups.len() * 2 + 1,
            PredefinedGroup::Internal => groups.len() * 2 + 2,
            PredefinedGroup::Parent => groups.len() * 2 + 3,
            PredefinedGroup::Sibling => groups.len() * 2 + 4,
            PredefinedGroup::Index => groups.len() * 2 + 5,
            PredefinedGroup::Object => groups.len() * 2 + 6,
            PredefinedGroup::Type => groups.len() * 2 + 7,
            PredefinedGroup::Unknown => groups.len() * 2 + 8,
        }
    }
}

/// Convert a string representation to a PredefinedGroup
fn group_from_str(group_str: &str) -> Option<PredefinedGroup> {
    match group_str {
        "builtin" => Some(PredefinedGroup::Builtin),
        "external" => Some(PredefinedGroup::External),
        "internal" => Some(PredefinedGroup::Internal),
        "parent" => Some(PredefinedGroup::Parent),
        "sibling" => Some(PredefinedGroup::Sibling),
        "index" => Some(PredefinedGroup::Index),
        "object" => Some(PredefinedGroup::Object),
        "type" => Some(PredefinedGroup::Type),
        "unknown" => Some(PredefinedGroup::Unknown),
        _ => None,
    }
}

/// Classifies the import source into predefined groups
fn classify_import_source(specifier: &str) -> PredefinedGroup {
    // List of Node.js builtin modules
    const BUILTIN_MODULES: &[&str] = &[
        "assert",
        "async",
        "async_hooks",
        "buffer",
        "child_process",
        "cluster",
        "console",
        "constants",
        "crypto",
        "dgram",
        "dns",
        "domain",
        "events",
        "fs",
        "http",
        "http2",
        "https",
        "inspector",
        "module",
        "net",
        "os",
        "path",
        "perf_hooks",
        "process",
        "punycode",
        "querystring",
        "readline",
        "repl",
        "stream",
        "string_decoder",
        "tls",
        "trace_events",
        "tty",
        "url",
        "util",
        "v8",
        "vm",
        "wasi",
        "worker_threads",
        "zlib",
    ];

    // Check for builtin modules
    if BUILTIN_MODULES.contains(&specifier) {
        return PredefinedGroup::Builtin;
    }

    // Check for relative paths that point to parent directory
    if specifier.starts_with("../") || specifier == ".." || specifier == "../" {
        return PredefinedGroup::Parent;
    }

    // Check for relative paths that point to current directory's index
    if specifier == "." || specifier == "./" || specifier == "./index" || specifier == "./index.js"
    {
        return PredefinedGroup::Index;
    }

    // Check for relative paths that point to current directory (siblings)
    if specifier.starts_with("./") {
        return PredefinedGroup::Sibling;
    }

    // Check for absolute paths or aliased paths (internal)
    if specifier.starts_with('/') || specifier.starts_with("~/") || specifier.starts_with("@/") {
        return PredefinedGroup::Internal;
    }

    // Everything else is considered external
    PredefinedGroup::External
}

/// Check if a source matches a pattern using glob syntax
fn matches_pattern(source: &str, pattern: &str) -> bool {
    // Handle regular glob patterns
    if pattern.contains('*') {
        let escaped = pattern.cow_replace('.', "\\.");
        let with_temp_stars = escaped.cow_replace("**", "__DOUBLE_STAR__");
        let with_single_stars = with_temp_stars.cow_replace('*', "[^/]*");
        let regex_pattern = with_single_stars.cow_replace("__DOUBLE_STAR__", ".*");
        return Regex::new(&format!("^{regex_pattern}$"))
            .map(|re| re.is_match(source))
            .unwrap_or(false);
    }

    // Exact match
    source == pattern
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // eslint test cases
        // Default order using require
        (
            r"
                        var fs = require('fs');
                        var async = require('async');
                        var relParent1 = require('../foo');
                        var relParent2 = require('../foo/bar');
                        var relParent3 = require('../');
                        var relParent4 = require('..');
                        var sibling = require('./foo');
                        var index = require('./');
                        ",
            None,
        ),
        // Default order using import
        (
            r"
                        import fs from 'fs';
                        import async, {foo1} from 'async';
                        import relParent1 from '../foo';
                        import relParent2, {foo2} from '../foo/bar';
                        import relParent3 from '../';
                        import sibling, {foo3} from './foo';
                        import index from './';
                        ",
            None,
        ),
        // Multiple module of the same rank next to each other
        (
            r"
                        var fs = require('fs');
                        var fs = require('fs');
                        var path = require('path');
                        var _ = require('lodash');
                        var async = require('async');
                        ",
            None,
        ),
        // Ignore dynamic requires
        (
            r"
                        var path = require('path');
                        var _ = require('lodash');
                        var async = require('async');
                        var fs = require('f' + 's');
                        ",
            None,
        ),
        // Ignore non-require call expressions
        (
            r"
                        var path = require('path');
                        var result = add(1, 2);
                        var _ = require('lodash');
                        ",
            None,
        ),
        // Ignore requires that are not at the top-level 1
        (
            r"
                        var index = require('./');
                        function foo() {
                            var fs = require('fs');
                        }
                        () => require('fs');
                        if (a) {
                            require('fs');
                        }
                        ",
            None,
        ),
        // Ignore requires that are not at the top-level 2
        (
            r"
                        const foo = [
                            require('./foo'),
                            require('fs'),
                        ];
                        ",
            None,
        ),
        // Ignore requires in template literal (1936)
        (r"const foo = `${require('./a')} ${require('fs')}`", None),
        // Ignore unknown/invalid cases
        (
            r"
                        var unknown1 = require('/unknown1');
                        var fs = require('fs');
                        var unknown2 = require('/unknown2');
                        var async = require('async');
                        var unknown3 = require('/unknown3');
                        var foo = require('../foo');
                        var unknown4 = require('/unknown4');
                        var bar = require('../foo/bar');
                        var unknown5 = require('/unknown5');
                        var parent = require('../');
                        var unknown6 = require('/unknown6');
                        var foo = require('./foo');
                        var unknown7 = require('/unknown7');
                        var index = require('./');
                        var unknown8 = require('/unknown8');
                        ",
            None,
        ),
        // Ignoring unassigned values by default (require)
        (
            r"
                        require('./foo');
                        require('fs');
                        var path = require('path');
                        ",
            None,
        ),
        // Ignoring unassigned values by default (import)
        (
            r"
                        import './foo';
                        import 'fs';
                        import path from 'path';
                        ",
            None,
        ),
        // No imports
        (
            r"
                        function add(a, b) {
                            return a + b;
                        }
                        var foo;
                        ",
            None,
        ),
        // Grouping import types
        (
            r"
                        var fs = require('fs');
                        var index = require('./');
                        var path = require('path');

                        var sibling = require('./foo');
                        var relParent3 = require('../');
                        var async = require('async');
                        var relParent1 = require('../foo');
                        ",
            Some(serde_json::json!([{
                "groups": [
                    ["builtin", "index"],
                    ["sibling", "parent", "external"]
                ]
            }])),
        ),
        // Grouping import types and alphabetize
        (
            r"
                        import async from 'async';
                        import fs from 'fs';
                        import path from 'path';

                        import relParent3 from '../';
                        import relParent1 from '../foo';
                        import sibling from './foo';
                        import index from '.';
                        ",
            Some(serde_json::json!([{
                "groups": [
                    ["builtin", "external"]
                ],
                "alphabetize": {
                    "order": "asc",
                    "caseInsensitive": true
                }
            }])),
        ),
        (
            r"
                        import { fooz } from '../baz.js'
                        import { foo } from './bar.js'
                        ",
            Some(serde_json::json!([{
                "alphabetize": {
                    "order": "asc",
                    "caseInsensitive": true
                },
                "groups": ["builtin", "external", "internal", ["parent", "sibling", "index"], "object"],
                "newlines-between": "always",
                "warnOnUnassignedImports": true
            }])),
        ),
        // Omitted types should implicitly be considered as the last type
        (
            r"
                        var index = require('./');
                        var path = require('path');
                        ",
            Some(serde_json::json!([{
                "groups": [
                    "index",
                    ["sibling", "parent", "external"]
                    // missing 'builtin'
                ]
            }])),
        ),
        // Mixing require and import should have import up top
        (
            r"
                        import async, {foo1} from 'async';
                        import relParent2, {foo2} from '../foo/bar';
                        import sibling, {foo3} from './foo';
                        var fs = require('fs');
                        var relParent1 = require('../foo');
                        var relParent3 = require('../');
                        var index = require('./');
                        ",
            None,
        ),
        // Manual test cases
        // Basic sorting
        (
            r"
            import fs from 'fs';
            import path from 'path';

            import chalk from 'chalk';
            import _ from 'lodash';

            import foo from '../foo';

            import bar from './bar';
            ",
            Some(serde_json::json!([{
                "groups": ["builtin", "external", "parent", "sibling", "index"],
                "newlines-between": "always"
            }])),
        ),
        // Alphabetical order
        (
            r"
            import a from 'a';
            import b from 'b';
            import c from 'c';
            ",
            Some(serde_json::json!([{
                "alphabetize": {
                    "order": "asc",
                    "caseInsensitive": true
                }
            }])),
        ),
        // Mixed groups with correct newlines
        (
            r"
            import fs from 'fs';
            import path from 'path';

            import _ from 'lodash';

            import foo from '../foo';
            import bar from './bar';
            ",
            Some(serde_json::json!([{
                "groups": ["builtin", "external", ["parent", "sibling"]],
                "newlines-between": "always"
            }])),
        ),
        // Test with pathGroups
        (
            r"
            import fs from 'fs';
            import _ from 'lodash';
            import MyComponent from '~/components/MyComponent';
            import utils from './utils';
            ",
            Some(serde_json::json!([{
                "groups": ["builtin", "external", "internal", "parent", "sibling", "index"],
                "pathGroups": [{
                    "pattern": "~/components/**",
                    "group": "internal",
                    "position": "after"
                }]
            }])),
        ),
    ];

    let fail = vec![
        // Wrong order
        (
            r"
            import _ from 'lodash';
            import fs from 'fs';
            ",
            Some(serde_json::json!([{
                "groups": ["builtin", "external"],
                "alphabetize": {
                    "order": "asc"
                }
            }])),
        ),
        // Missing newline between groups
        // (
        //     r"
        //     import fs from 'fs';
        //     import _ from 'lodash';  // Should have newline before this
        //     ",
        //     Some(serde_json::json!([{
        //         "groups": ["builtin", "external"],
        //         "newlines-between": "always"
        //     }])),
        // ),
        // Wrong alphabetical order
        (
            r"
            import b from 'b';
            import a from 'a';
            ",
            Some(serde_json::json!([{
                "alphabetize": {
                    "order": "asc"
                }
            }])),
        ),
        (
            r"
            import MyComponent from '~/components/MyComponent';
            import _ from 'lodash';
            ",
            Some(serde_json::json!([{
                "groups": ["builtin", "external", "internal"],
                "pathGroups": [{
                    "pattern": "~/components/**",
                    "group": "internal",
                    "position": "after"
                }]
            }])),
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_classify_import_source() {
        // Test builtin modules
        assert_eq!(classify_import_source("fs"), PredefinedGroup::Builtin);
        assert_eq!(classify_import_source("path"), PredefinedGroup::Builtin);

        // Test external modules
        assert_eq!(classify_import_source("lodash"), PredefinedGroup::External);
        assert_eq!(classify_import_source("@org/package"), PredefinedGroup::External);

        // Test parent paths
        assert_eq!(classify_import_source(".."), PredefinedGroup::Parent);
        assert_eq!(classify_import_source("../"), PredefinedGroup::Parent);
        assert_eq!(classify_import_source("../foo"), PredefinedGroup::Parent);

        // Test sibling paths
        assert_eq!(classify_import_source("./foo"), PredefinedGroup::Sibling);
        // Test index paths
        assert_eq!(classify_import_source("./"), PredefinedGroup::Index);
    }
}
