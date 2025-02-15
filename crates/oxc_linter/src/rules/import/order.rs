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
    Builtin = 0,
    External = 1,
    Internal = 2,
    Parent = 3,
    Sibling = 4,
    Index = 5,
    Object = 6,
}

impl std::cmp::Ord for PredefinedGroup {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        get_predefined_group_rank(self).cmp(&get_predefined_group_rank(other))
    }
}

impl std::cmp::PartialOrd for PredefinedGroup {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::PartialEq for PredefinedGroup {
    fn eq(&self, other: &Self) -> bool {
        get_predefined_group_rank(self) == get_predefined_group_rank(other)
    }
}

impl std::cmp::Eq for PredefinedGroup {}

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
    /// 0 => builtin, 1 => external (in this tiny example)
    group: PredefinedGroup,
    /// the import's position in the file as encountered
    original_index: usize,
    /// for generating a diagnostic
    span: Span,
    /// the import source, e.g. "fs" or "./bar"
    specifier: String,
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

    fn run_once(&self, ctx: &LintContext<'_>) {
        // Gather all imports from the module record
        let module_record = ctx.module_record();
        let import_entries = &module_record.import_entries;

        // Convert each import entry into our SimpleImport struct
        let imports: Vec<ImportInfo> = import_entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let specifier = entry.module_request.name();
                ImportInfo {
                    group: classify_import_source(specifier),
                    original_index: idx,
                    span: entry.statement_span,
                    specifier: specifier.to_owned(),
                }
            })
            .collect();

        println!("Imports: {:#?}", imports);

        // Make a sorted clone to see what the ideal order “should” be
        let mut sorted_imports = imports.clone();

        // Sort by group first (builtin vs. external),
        // then alphabetically by specifier as a secondary key
        sorted_imports.sort_by(|a, b| {
            // First compare groups
            let group_cmp = a.group.cmp(&b.group);
            if group_cmp != std::cmp::Ordering::Equal {
                return group_cmp;
            }

            // If they're in the same group, compare by specifier
            a.specifier.cmp(&b.specifier)
        });

        // Compare actual vs. sorted order:
        // - If any import is in a different position, emit a diagnostic
        for (actual, ideal) in imports.iter().zip(sorted_imports.iter()) {
            if actual.specifier != ideal.specifier {
                ctx.diagnostic(
                    OxcDiagnostic::error(format!(
                        "Import {:?} should appear after {:?}",
                        actual.specifier, ideal.specifier
                    ))
                    .with_label(actual.span),
                );
            }
        }
    }
}

/// Classifies the import source as builtin or external
/// In a real rule, you'd detect core modules, node_modules, local paths, etc.
fn classify_import_source(specifier: &str) -> PredefinedGroup {
    match specifier {
        "assert" | "async" | "buffer" | "child_process" | "cluster" | "crypto" | "dgram"
        | "dns" | "domain" | "events" | "fs" | "http" | "https" | "net" | "os" | "path"
        | "punycode" | "querystring" | "readline" | "stream" | "string_decoder" | "tls" | "tty"
        | "url" | "util" | "v8" | "vm" | "zlib" => PredefinedGroup::Builtin, // builtin
        path if path.starts_with("./") => PredefinedGroup::Sibling,
        path if path.starts_with("../") => PredefinedGroup::Parent,
        "." => PredefinedGroup::Parent,
        _ => PredefinedGroup::External, // external
    }
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
                        import async, {foo1} from 'async';
                        import fs from 'fs';
                        import relParent3 from '../';
                        import relParent1 from '../foo';
                        import relParent2, {foo2} from '../foo/bar';
                        import index from './';
                        import sibling, {foo3} from './foo';
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

                        import index from '.';
                        import relParent3 from '../';
                        import relParent1 from '../foo';
                        import sibling from './foo';
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
                "groups": ["builtin", "external"]
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
    fn test_predefined_group_ordering() {
        // Test basic ordering
        assert!(PredefinedGroup::Builtin < PredefinedGroup::External);
        assert!(PredefinedGroup::External < PredefinedGroup::Internal);
        assert!(PredefinedGroup::Internal < PredefinedGroup::Parent);
        assert!(PredefinedGroup::Parent < PredefinedGroup::Sibling);
        assert!(PredefinedGroup::Sibling < PredefinedGroup::Index);
        assert!(PredefinedGroup::Index < PredefinedGroup::Object);

        // Test equality
        assert_eq!(PredefinedGroup::Builtin, PredefinedGroup::Builtin);
        assert_eq!(PredefinedGroup::External, PredefinedGroup::External);

        // Test transitivity
        assert!(PredefinedGroup::Builtin < PredefinedGroup::Internal);
        assert!(PredefinedGroup::Internal < PredefinedGroup::Sibling);
        assert!(PredefinedGroup::Builtin < PredefinedGroup::Sibling);
    }

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
        assert_eq!(classify_import_source("./"), PredefinedGroup::Sibling);
    }
}
