use std::{
    borrow::Cow,
    fmt::{Display, Write},
    str::FromStr,
};

use cow_utils::CowUtils;
use itertools::Itertools;
use oxc_ast::{
    ast::{ImportDeclaration, ImportDeclarationSpecifier, Statement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn unexpected_syntax_order_diagnostic(
    curr_kind: &ImportKind,
    prev_kind: &ImportKind,
    span2: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Expected '{curr_kind}' syntax before '{prev_kind}' syntax."))
        .with_label(span2)
}

fn sort_imports_alphabetically_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Imports should be sorted alphabetically.").with_label(span)
}

fn sort_members_alphabetically_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Member '{name}' of the import declaration should be sorted alphabetically."
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct SortImports(Box<SortImportsOptions>);

#[derive(Debug, Default, Clone)]
pub struct SortImportsOptions {
    ignore_case: bool,
    ignore_declaration_sort: bool,
    ignore_member_sort: bool,
    allow_separated_groups: bool,
    member_syntax_sort_order: MemberSyntaxSortOrder,
}

impl std::ops::Deref for SortImports {
    type Target = SortImportsOptions;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule checks all import declarations and verifies that all imports are first sorted
    /// by the used member syntax and then alphabetically by the first member or alias name.
    ///
    /// When declaring multiple imports, a sorted list of import declarations make it easier for developers to read
    /// the code and find necessary imports later.
    ///
    /// ### Why is this bad?
    ///
    /// ### Example
    /// ```javascript
    /// import {b, a, c} from 'foo.js'
    ///
    /// import d from 'foo.js';
    /// import e from 'bar.js';
    /// ```
    SortImports,
    style,
    conditional_fix
);

impl Rule for SortImports {
    fn from_configuration(value: serde_json::Value) -> Self {
        let Some(config) = value.get(0) else {
            return Self(Box::default());
        };

        let ignore_case =
            config.get("ignoreCase").and_then(serde_json::Value::as_bool).unwrap_or_default();
        let ignore_member_sort =
            config.get("ignoreMemberSort").and_then(serde_json::Value::as_bool).unwrap_or_default();
        let ignore_declaration_sort = config
            .get("ignoreDeclarationSort")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or_default();
        let allow_separated_groups = config
            .get("allowSeparatedGroups")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or_default();

        let member_syntax_sort_order = config
            .get("memberSyntaxSortOrder")
            .and_then(|v| v.as_array())
            .map(|arr| {
                // memberSyntaxSortOrder in config file must have 4 items
                if arr.len() != 4 {
                    return MemberSyntaxSortOrder::default();
                }

                let kinds: Vec<ImportKind> = arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(ImportKind::from_str)
                    .filter_map(Result::ok)
                    .unique()
                    .collect();

                // 4 items must all unique and valid.
                if kinds.len() != 4 {
                    return MemberSyntaxSortOrder::default();
                }

                MemberSyntaxSortOrder(kinds)
            })
            .unwrap_or_default();

        Self(Box::new(SortImportsOptions {
            ignore_case,
            ignore_declaration_sort,
            ignore_member_sort,
            allow_separated_groups,
            member_syntax_sort_order,
        }))
    }

    fn run_once(&self, ctx: &LintContext) {
        let Some(root) = ctx.nodes().root_node() else {
            return;
        };
        let AstKind::Program(program) = root.kind() else { unreachable!() };

        let mut import_declarations = vec![];

        for statement in &program.body {
            if let Statement::ImportDeclaration(decl) = statement {
                import_declarations.push(decl);
            } else {
                break;
            }
        }

        let mut previous: Option<&ImportDeclaration> = None;

        for current in import_declarations {
            if !self.ignore_declaration_sort {
                // ```js
                // import b from 'foo.js'
                //
                // import a from 'foo.js'
                // ```
                // when visit line 3, line 2 will make `previous` to be `None`
                if self.allow_separated_groups
                    && previous.is_some_and(|previous| {
                        get_number_of_lines_between(previous.span, current.span, ctx) > 0
                    })
                {
                    previous = None;
                }

                if let Some(previous) = previous {
                    self.check_syntax_order_and_member_order(previous, current, ctx);
                }

                previous = Some(current);
            }

            if !self.ignore_member_sort {
                self.check_member_sort_with_fix(current, ctx);
            }
        }
    }
}

impl SortImports {
    // Check between two import declarations.
    //
    // 1. syntax order
    // ```js
    // import a from 'foo.js' // single
    // import from 'foo.js' // none
    // ```
    //
    // 2. member order. If the syntax order is the same, check the member order.
    //
    // ```js
    // import { b } from 'foo.js'
    // import { c } from 'foo.js'
    // ```
    fn check_syntax_order_and_member_order(
        &self,
        previous: &ImportDeclaration,
        current: &ImportDeclaration,
        ctx: &LintContext,
    ) {
        let current_member_syntax_group_index =
            self.member_syntax_sort_order.get_group_index_by_import_decl(current);
        let previous_member_syntax_group_index =
            self.member_syntax_sort_order.get_group_index_by_import_decl(previous);

        let mut current_local_member_name = get_first_local_member_name(current);
        let mut previous_local_member_name = get_first_local_member_name(previous);

        if self.ignore_case {
            current_local_member_name = current_local_member_name
                .map(|name| Cow::Owned(name.cow_to_lowercase().into_owned()));
            previous_local_member_name = previous_local_member_name
                .map(|name| Cow::Owned(name.cow_to_lowercase().into_owned()));
        }

        // "memberSyntaxSortOrder": ["none", "all", "multiple", "single"]
        // ```js
        // import a from 'foo.js'
        // import from 'foo.js' // <-- incorrect, 'none' should come before 'single'
        // ```
        match current_member_syntax_group_index.cmp(&previous_member_syntax_group_index) {
            std::cmp::Ordering::Less => {
                let current_kind =
                    self.member_syntax_sort_order.get(current_member_syntax_group_index);
                let previous_kind =
                    self.member_syntax_sort_order.get(previous_member_syntax_group_index);
                if let Some((current_kind, previous_kind)) = current_kind.zip(previous_kind) {
                    ctx.diagnostic(unexpected_syntax_order_diagnostic(
                        current_kind,
                        previous_kind,
                        current.span,
                    ));
                }
            }
            std::cmp::Ordering::Equal => {
                // ```js
                // import { b } from 'foo.js'
                // import { a } from 'foo.js' // <-- incorrect, 'a' should come before 'b'
                // ```
                if let Some((current_name, previous_name)) =
                    current_local_member_name.zip(previous_local_member_name)
                {
                    if current_name < previous_name {
                        ctx.diagnostic(sort_imports_alphabetically_diagnostic(current.span));
                    }
                }
            }
            std::cmp::Ordering::Greater => {}
        }
    }

    // Check member sort in a import declaration
    // ```js
    // import { b, a } from 'foo.js'
    // ```
    fn check_member_sort_with_fix(&self, current: &ImportDeclaration, ctx: &LintContext) {
        let Some(specifiers) = &current.specifiers else {
            return;
        };

        let specifiers: Vec<_> = specifiers
            .iter()
            .filter_map(|specifier| {
                if let ImportDeclarationSpecifier::ImportSpecifier(ref specifier) = specifier {
                    Some(specifier)
                } else {
                    None
                }
            })
            .collect();

        if specifiers.len() < 2 {
            return;
        }

        let unsorted = specifiers
            .windows(2)
            .find(|window| {
                let a = window[0].local.name.as_str();
                let b = window[1].local.name.as_str();

                if self.ignore_case {
                    a.cow_to_lowercase() > b.cow_to_lowercase()
                } else {
                    a > b
                }
            })
            .map(|window| (window[1].local.name.as_str(), window[1].span));

        let Some((unsorted_name, unsorted_span)) = unsorted else {
            return;
        };

        // ESLint check if there are comments in the ImportSpecifier list, and don't rearrange the specifiers.
        // We don't have a direct way to check if there are comments in the specifiers(may need lookahead/lookbehind source text).
        // ```js
        // // this is not fixable in ESLint
        // import { /* comment */ a, b, c, d } from 'foo.js'
        // ```
        // I use ImportStatement's span to check if there are comments between the specifiers.
        let is_fixable = !ctx.semantic().has_comments_between(current.span);

        if is_fixable {
            // Safe to index because we know that `specifiers` is at least 2 element long
            let specifiers_span = specifiers[0].span.merge(&specifiers[specifiers.len() - 1].span);
            ctx.diagnostic_with_fix(
                sort_members_alphabetically_diagnostic(unsorted_name, unsorted_span),
                |fixer| {
                    // import { a, b,      c, d } from 'foo.js'
                    //            ^  ^^^^^^  ^
                    let mut paddings: Vec<&str> = specifiers
                        .windows(2)
                        .map(|window| {
                            let a = window[0].span;
                            let b = window[1].span;

                            let padding = Span::new(a.end, b.start);
                            ctx.source_range(padding)
                        })
                        .collect();

                    // add a empty string for zip with specifiers
                    paddings.push("");

                    let specifiers = specifiers.iter().sorted_by(|a, b| {
                        let a = a.local.name.as_str();
                        let b = b.local.name.as_str();

                        if self.ignore_case {
                            a.cow_to_lowercase().cmp(&b.cow_to_lowercase())
                        } else {
                            a.cmp(b)
                        }
                    });

                    let sorted_text = specifiers.zip(paddings).fold(
                        String::new(),
                        |mut acc, (specifier, padding)| {
                            let _ = acc.write_str(ctx.source_range(specifier.span));
                            let _ = acc.write_str(padding);
                            acc
                        },
                    );

                    fixer.replace(specifiers_span, sorted_text)
                },
            );
        } else {
            ctx.diagnostic(sort_members_alphabetically_diagnostic(unsorted_name, unsorted_span));
        }
    }
}

#[derive(Debug, Clone)]
struct MemberSyntaxSortOrder(Vec<ImportKind>);

impl Default for MemberSyntaxSortOrder {
    fn default() -> Self {
        MemberSyntaxSortOrder(vec![
            ImportKind::None,
            ImportKind::All,
            ImportKind::Multiple,
            ImportKind::Single,
        ])
    }
}

impl std::ops::Deref for MemberSyntaxSortOrder {
    type Target = Vec<ImportKind>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl MemberSyntaxSortOrder {
    fn get_group_index_by_import_decl(&self, decl: &ImportDeclaration) -> usize {
        // import "foo.js" -> ImportKind::None
        // import * as foo from "foo.js" -> ImportKind::All
        // import { a, b } from "foo.js" -> ImportKind::Multiple
        // import a from "foo.js" -> ImportKind::Single
        // import { a } from 'foo.js' -> ImportKind::Single
        let import_kind = match &decl.specifiers {
            Some(specifiers) => {
                if specifiers.is_empty() {
                    ImportKind::None
                } else if specifiers.len() == 1 {
                    if matches!(
                        specifiers[0],
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(_)
                    ) {
                        ImportKind::All
                    } else {
                        ImportKind::Single
                    }
                } else {
                    ImportKind::Multiple
                }
            }
            None => ImportKind::None,
        };

        self.iter().position(|kind| kind == &import_kind).unwrap_or_default()
    }
}

#[derive(Debug, Default, Clone, Hash, Eq, PartialEq)]
enum ImportKind {
    // import from 'foo.js'
    #[default]
    None,
    // import * from 'foo.js'
    All,
    // import { a, b } from 'foo.js'
    Multiple,
    // import a from 'foo.js'
    Single,
}

impl FromStr for ImportKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(ImportKind::None),
            "all" => Ok(ImportKind::All),
            "multiple" => Ok(ImportKind::Multiple),
            "single" => Ok(ImportKind::Single),
            _ => Err("Invalid import kind"),
        }
    }
}

impl Display for ImportKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportKind::None => write!(f, "None"),
            ImportKind::All => write!(f, "All"),
            ImportKind::Multiple => write!(f, "Multiple"),
            ImportKind::Single => write!(f, "Single"),
        }
    }
}

fn get_first_local_member_name<'a>(decl: &ImportDeclaration<'a>) -> Option<Cow<'a, str>> {
    let specifiers = decl.specifiers.as_ref()?;
    specifiers.first().map(ImportDeclarationSpecifier::name)
}

// Calculates number of lines between two nodes. It is assumed that the given `left` span appears before
// the given `right` span in the source code. Lines are counted from the end of the `left` span till the
// start of the `right` span. If the given span are on the same line, or `right` span is appears before `left` span,
// it returns `0`.
fn get_number_of_lines_between(left: Span, right: Span, ctx: &LintContext) -> usize {
    if left.end >= right.start {
        return 0;
    }
    let between_span = Span::new(left.end, right.start);
    let count = ctx.source_range(between_span).lines().count();

    // In same line
    if count < 2 {
        return 0;
    }

    // In different lines, need to subtract 2 because the count includes the first and last line.
    count - 2
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "import a from 'foo.js';
            import b from 'bar.js';
            import c from 'baz.js';
            ",
            None,
        ),
        (
            "import * as B from 'foo.js';
            import A from 'bar.js';",
            None,
        ),
        (
            "import * as B from 'foo.js';
            import {a, b} from 'bar.js';",
            None,
        ),
        (
            "import {b, c} from 'bar.js';
            import A from 'foo.js';",
            None,
        ),
        (
            "import A from 'bar.js';
            import {b, c} from 'foo.js';",
            Some(
                serde_json::json!([{ "memberSyntaxSortOrder": ["single", "multiple", "none", "all"] }]),
            ),
        ),
        (
            "import {a, b} from 'bar.js';
            import {c, d} from 'foo.js';",
            None,
        ),
        (
            "import A from 'foo.js';
            import B from 'bar.js';",
            None,
        ),
        (
            "import A from 'foo.js';
            import a from 'bar.js';",
            None,
        ),
        (
            "import a, * as b from 'foo.js';
            import c from 'bar.js';",
            None,
        ),
        (
            "import 'foo.js';
            import a from 'bar.js';",
            None,
        ),
        (
            "import B from 'foo.js';
            import a from 'bar.js';",
            None,
        ),
        (
            "import a from 'foo.js';
            import B from 'bar.js';",
            Some(serde_json::json!([{ "ignoreCase": true }])),
        ),
        ("import {a, b, c, d} from 'foo.js';", None),
        (
            "import a from 'foo.js';
            import B from 'bar.js';",
            Some(serde_json::json!([{ "ignoreDeclarationSort": true }])),
        ),
        (
            "import {b, A, C, d} from 'foo.js';",
            Some(serde_json::json!([{ "ignoreMemberSort": true }])),
        ),
        (
            "import {B, a, C, d} from 'foo.js';",
            Some(serde_json::json!([{ "ignoreMemberSort": true }])),
        ),
        ("import {a, B, c, D} from 'foo.js';", Some(serde_json::json!([{ "ignoreCase": true }]))),
        ("import a, * as b from 'foo.js';", None),
        (
            "import * as a from 'foo.js';

            import b from 'bar.js';",
            None,
        ),
        (
            "import * as bar from 'bar.js';
            import * as foo from 'foo.js';",
            None,
        ),
        (
            "import 'foo';
            import bar from 'bar';",
            Some(serde_json::json!([{ "ignoreCase": true }])),
        ),
        ("import React, {Component} from 'react';", None),
        (
            "import b from 'b';

            import a from 'a';",
            Some(serde_json::json!([{ "allowSeparatedGroups": true }])),
        ),
        (
            "import a from 'a';

            import 'b';",
            Some(serde_json::json!([{ "allowSeparatedGroups": true }])),
        ),
        (
            "import { b } from 'b';


            import { a } from 'a';",
            Some(serde_json::json!([{ "allowSeparatedGroups": true }])),
        ),
        (
            "import b from 'b';
            // comment
            import a from 'a';",
            Some(serde_json::json!([{ "allowSeparatedGroups": true }])),
        ),
        (
            "import b from 'b';
            foo();
            import a from 'a';",
            Some(serde_json::json!([{ "allowSeparatedGroups": true }])),
        ),
        (
            "import { b } from 'b';/*
                comment
            */import { a } from 'a';",
            Some(serde_json::json!([{ "allowSeparatedGroups": true }])),
        ),
        (
            "import b from
            'b';

            import
                a from 'a';",
            Some(serde_json::json!([{ "allowSeparatedGroups": true }])),
        ),
        (
            "import c from 'c';

            import a from 'a';
            import b from 'b';",
            Some(serde_json::json!([{ "allowSeparatedGroups": true }])),
        ),
        (
            "import c from 'c';

            import b from 'b';

            import a from 'a';",
            Some(serde_json::json!([{ "allowSeparatedGroups": true }])),
        ),
    ];

    let fail = vec![
        (
            "import a from 'foo.js';
            import A from 'bar.js';",
            None,
        ),
        (
            "import b from 'foo.js';
            import a from 'bar.js';",
            None,
        ),
        (
            "import {b, c} from 'foo.js';
            import {a, d} from 'bar.js';",
            None,
        ),
        (
            "import * as foo from 'foo.js';
            import * as bar from 'bar.js';",
            None,
        ),
        (
            "import a from 'foo.js';
            import {b, c} from 'bar.js';",
            None,
        ),
        (
            "import a from 'foo.js';
            import * as b from 'bar.js';",
            None,
        ),
        (
            "import a from 'foo.js';
            import 'bar.js';",
            None,
        ),
        (
            "import b from 'bar.js';
            import * as a from 'foo.js';",
            Some(
                serde_json::json!([{ "memberSyntaxSortOrder": ["all", "single", "multiple", "none"] }]),
            ),
        ),
        ("import {b, a, d, c} from 'foo.js';", None),
        (
            "import {b, a, d, c} from 'foo.js';
            import {e, f, g, h} from 'bar.js';",
            Some(serde_json::json!([{ "ignoreDeclarationSort": true }])),
        ),
        ("import {a, B, c, D} from 'foo.js';", None),
        ("import {zzzzz, /* comment */ aaaaa} from 'foo.js';", None),
        ("import {zzzzz /* comment */, aaaaa} from 'foo.js';", None),
        ("import {/* comment */ zzzzz, aaaaa} from 'foo.js';", None),
        ("import {zzzzz, aaaaa /* comment */} from 'foo.js';", None),
        (
            "
                import {
                boop,
                foo,
                zoo,
                baz as qux,
                bar,
                beep
                } from 'foo.js';
            ",
            None,
        ),
        (
            "import b from 'b';
            import a from 'a';",
            None,
        ),
        (
            "import b from 'b';
            import a from 'a';",
            Some(serde_json::json!([{}])),
        ),
        (
            "import b from 'b';
            import a from 'a';",
            Some(serde_json::json!([{ "allowSeparatedGroups": false }])),
        ),
        (
            "import b from 'b';import a from 'a';",
            Some(serde_json::json!([{ "allowSeparatedGroups": false }])),
        ),
        (
            "import b from 'b'; /* comment */ import a from 'a';",
            Some(serde_json::json!([{ "allowSeparatedGroups": false }])),
        ),
        (
            "import b from 'b'; // comment
            import a from 'a';",
            Some(serde_json::json!([{ "allowSeparatedGroups": false }])),
        ),
        (
            "import b from 'b'; // comment 1
            /* comment 2 */import a from 'a';",
            Some(serde_json::json!([{ "allowSeparatedGroups": false }])),
        ),
        (
            "import { b } from 'b'; /* comment line 1
                comment line 2 */ import { a } from 'a';",
            Some(serde_json::json!([{ "allowSeparatedGroups": false }])),
        ),
        (
            "import b
            from 'b'; import a
            from 'a';",
            Some(serde_json::json!([{ "allowSeparatedGroups": false }])),
        ),
        (
            "import { b } from
            'b'; /* comment */ import
             { a } from 'a';",
            Some(serde_json::json!([{ "allowSeparatedGroups": false }])),
        ),
        (
            "import { b } from
            'b';
            import
                { a } from 'a';",
            Some(serde_json::json!([{ "allowSeparatedGroups": false }])),
        ),
        (
            "import c from 'c';

            import b from 'b';
            import a from 'a';",
            Some(serde_json::json!([{ "allowSeparatedGroups": true }])),
        ),
        (
            "import b from 'b';

            import { c, a } from 'c';",
            Some(serde_json::json!([{ "allowSeparatedGroups": true }])),
        ),
    ];

    let fix = vec![
        ("import {b, a, d, c} from 'foo.js';", "import {a, b, c, d} from 'foo.js';", None),
        ("import {a, B, c, D} from 'foo.js';", "import {B, D, a, c} from 'foo.js';", None),
        (
            " import {
                boop,
                foo,
                zoo,
                baz as qux,
                bar,
                beep
              } from 'foo.js';",
            " import {
                bar,
                beep,
                boop,
                foo,
                baz as qux,
                zoo
              } from 'foo.js';",
            None,
        ),
        (
            "
              import b from 'b';

              import { c, a } from 'c';",
            "
              import b from 'b';

              import { a, c } from 'c';",
            None,
        ),
        // Not fixed due to comment
        (
            "import {zzzzz, /* comment */ aaaaa} from 'foo.js';",
            "import {zzzzz, /* comment */ aaaaa} from 'foo.js';",
            None,
        ),
        (
            "import {zzzzz /* comment */, aaaaa} from 'foo.js';",
            "import {zzzzz /* comment */, aaaaa} from 'foo.js';",
            None,
        ),
        (
            "import {/* comment */ zzzzz, aaaaa} from 'foo.js';",
            "import {/* comment */ zzzzz, aaaaa} from 'foo.js';",
            None,
        ),
        (
            "import {zzzzz, aaaaa /* comment */} from 'foo.js';",
            "import {zzzzz, aaaaa /* comment */} from 'foo.js';",
            None,
        ),
    ];
    Tester::new(SortImports::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
