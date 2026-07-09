use std::{borrow::Cow, cell::OnceCell};

use itertools::Itertools;
use oxc_ast::{
    AstKind,
    ast::{ImportDeclaration, ImportDeclarationSpecifier},
};
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::{FxHashMap, FxHashSet};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    module_record::{ImportImportName, RequestedModule},
    rule::{DefaultRuleConfig, Rule},
};

fn no_duplicates_diagnostic<I>(
    module_name: &str,
    first_import: Span,
    other_imports: I,
) -> OxcDiagnostic
where
    I: IntoIterator<Item = Span>,
{
    const MAX_MODULE_LEN: usize = 16;

    let message = if module_name.len() > MAX_MODULE_LEN {
        Cow::Borrowed("Modules should not be imported multiple times in the same file")
    } else {
        Cow::Owned(format!("Module '{module_name}' is imported more than once in this file"))
    };
    let labels = std::iter::once(first_import.primary_label("It is first imported here"))
        .chain(other_imports.into_iter().map(LabeledSpan::underline));

    OxcDiagnostic::warn(message)
        .with_labels(labels)
        .with_help("Merge these imports into a single import statement")
}

// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/no-duplicates.md>
#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoDuplicates {
    /// When set to `true`, prefer inline type imports instead of separate type import
    /// statements for TypeScript code.
    ///
    /// Examples of **correct** code with this option set to `true`:
    /// ```typescript
    /// import { Foo, type Bar } from './module';
    /// ```
    #[serde(alias = "prefer-inline")]
    prefer_inline: bool,

    /// When set to `true`, the rule will consider the query string part of the import path
    /// when determining if imports are duplicates. This is useful when using loaders like
    /// webpack that use query strings to configure how a module should be loaded.
    ///
    /// Examples of **correct** code with this option set to `true`:
    /// ```javascript
    /// import x from './bar?optionX';
    /// import y from './bar?optionY';
    /// ```
    consider_query_string: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports if a resolved path is imported more than once in the same module.
    /// This helps avoid unnecessary duplicate imports and keeps the code clean.
    ///
    /// ### Why is this bad?
    ///
    /// Importing the same module multiple times can lead to redundancy and
    /// unnecessary complexity. It also affects maintainability, as it might
    /// confuse developers and result in inconsistent usage of imports across the code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import { foo } from './module';
    /// import { bar } from './module';
    ///
    /// import a from './module';
    /// import { b } from './module';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { foo, bar } from './module';
    ///
    /// import * as a from 'foo'; // separate statements for namespace imports
    /// import { b } from 'foo';
    ///
    /// import { c } from 'foo';      // separate type imports, unless
    /// import type { d } from 'foo'; // `prefer-inline` is true
    /// ```
    NoDuplicates,
    import,
    style,
    conditional_fix,
    config = NoDuplicates,
    version = "0.2.11",
    short_description = "Forbid importing the same module multiple times.",
);

impl Rule for NoDuplicates {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once<'a>(&self, ctx: &LintContext<'a>) {
        let module_record = ctx.module_record();

        // Map each import statement span to its declaration node so the fixer can inspect
        // specifiers, default names and brace positions when merging duplicates.
        let import_declarations: OnceCell<FxHashMap<Span, &'a ImportDeclaration<'a>>> =
            OnceCell::new();

        let groups = module_record
            .requested_modules
            .iter()
            .map(|(source, requested_modules)| {
                let resolved_absolute_path = module_record.get_loaded_module(source).map_or_else(
                    || source.to_string(),
                    |module| module.resolved_absolute_path.to_string_lossy().to_string(),
                );
                // When consider_query_string is true, include the query string in the grouping key.
                // When false (default), strip query strings so imports with different query strings
                // are grouped together as duplicates.
                let grouping_key = if self.consider_query_string {
                    // Include query string from the original source
                    if let Some(query_pos) = source.as_str().find('?') {
                        format!("{}{}", resolved_absolute_path, &source.as_str()[query_pos..])
                    } else {
                        resolved_absolute_path
                    }
                } else {
                    resolved_absolute_path
                };
                (grouping_key, requested_modules)
            })
            .chunk_by(|r| r.0.clone());

        for (_path, group) in &groups {
            let requested_modules = group
                .into_iter()
                .flat_map(|(_path, requested_modules)| requested_modules)
                .filter(|requested_module| requested_module.is_import);
            // When prefer_inline is false, 0 is value, 1 is type named, 2 is type namespace and 3 is type default
            // When prefer_inline is true, 0 is value and type named, 2 is type // namespace and 3 is type default
            let mut import_entries_maps: FxHashMap<u8, Vec<&RequestedModule>> =
                FxHashMap::default();
            // Side-effect and top-level type-only imports cannot be merged directly. A runtime
            // import with specifiers (including inline type specifiers) can absorb either.
            let mut key0_side_effect_imports = Vec::new();
            let mut key0_type_only_imports = Vec::new();
            let mut key0_has_runtime_specifiers = false;
            for requested_module in requested_modules {
                let import_entries = module_record
                    .import_entries
                    .iter()
                    .filter(|entry| entry.module_request.span == requested_module.span)
                    .collect::<Vec<_>>();
                if import_entries.is_empty() {
                    let key = u8::from(requested_module.is_type && !self.prefer_inline);
                    import_entries_maps.entry(key).or_default().push(requested_module);
                    if key == 0 {
                        if requested_module.is_type {
                            key0_type_only_imports.push(requested_module);
                        } else {
                            key0_side_effect_imports.push(requested_module);
                        }
                    }
                    continue;
                }
                let mut flags = [true; 4];
                let mut pushed_to_key0 = false;
                for import_entry in import_entries {
                    let key = if import_entry.is_type {
                        match import_entry.import_name {
                            ImportImportName::Name(_) => u8::from(!self.prefer_inline),
                            ImportImportName::NamespaceObject => 2,
                            ImportImportName::Default(_) => 3,
                        }
                    } else {
                        match import_entry.import_name {
                            ImportImportName::NamespaceObject => 2,
                            _ => 0,
                        }
                    };

                    if flags[key as usize] {
                        flags[key as usize] = false;
                        import_entries_maps.entry(key).or_default().push(requested_module);
                        if key == 0 {
                            pushed_to_key0 = true;
                        }
                    }
                }
                if pushed_to_key0 {
                    if requested_module.is_type {
                        key0_type_only_imports.push(requested_module);
                    } else {
                        key0_has_runtime_specifiers = true;
                    }
                }
            }

            for i in 0..4 {
                if i == 0 && !key0_has_runtime_specifiers {
                    check_duplicates(
                        ctx,
                        &import_declarations,
                        self.prefer_inline,
                        Some(&key0_side_effect_imports),
                    );
                    check_duplicates(
                        ctx,
                        &import_declarations,
                        self.prefer_inline,
                        Some(&key0_type_only_imports),
                    );
                    continue;
                }
                check_duplicates(
                    ctx,
                    &import_declarations,
                    self.prefer_inline,
                    import_entries_maps.get(&i),
                );
            }
        }
    }
}

fn build_import_declarations<'a>(
    ctx: &LintContext<'a>,
) -> FxHashMap<Span, &'a ImportDeclaration<'a>> {
    ctx.nodes()
        .iter()
        .filter_map(|node| match node.kind() {
            AstKind::ImportDeclaration(decl) => Some((decl.span, decl)),
            _ => None,
        })
        .collect()
}

fn check_duplicates<'a>(
    ctx: &LintContext<'a>,
    import_declarations: &OnceCell<FxHashMap<Span, &'a ImportDeclaration<'a>>>,
    prefer_inline: bool,
    requested_modules: Option<&Vec<&RequestedModule>>,
) {
    let Some(requested_modules) = requested_modules else { return };
    if requested_modules.len() <= 1 {
        return;
    }

    let mut labels = requested_modules.iter().map(|m| m.span);
    let first = labels.next().unwrap(); // we know there is at least one
    let module_name = ctx.source_range(first).trim_matches('\'').trim_matches('"');
    let diagnostic = no_duplicates_diagnostic(module_name, first, labels);

    let import_declarations = import_declarations.get_or_init(|| build_import_declarations(ctx));
    let decls: Vec<&ImportDeclaration<'a>> = requested_modules
        .iter()
        .filter_map(|m| import_declarations.get(&m.statement_span).copied())
        .collect();

    if decls.len() == requested_modules.len() {
        ctx.diagnostic_with_fix(diagnostic, |fixer| {
            merge_imports_fix(fixer, ctx, prefer_inline, &decls)
        });
    } else {
        ctx.diagnostic(diagnostic);
    }
}

fn has_namespace(decl: &ImportDeclaration) -> bool {
    decl.specifiers.as_ref().is_some_and(|specifiers| {
        specifiers
            .iter()
            .any(|s| matches!(s, ImportDeclarationSpecifier::ImportNamespaceSpecifier(_)))
    })
}

fn has_named_specifiers(decl: &ImportDeclaration) -> bool {
    decl.specifiers.as_ref().is_some_and(|specifiers| {
        specifiers.iter().any(|s| matches!(s, ImportDeclarationSpecifier::ImportSpecifier(_)))
    })
}

fn default_import_name<'a>(decl: &'a ImportDeclaration<'a>) -> Option<&'a str> {
    decl.specifiers.as_ref().and_then(|specifiers| {
        specifiers.iter().find_map(|s| match s {
            ImportDeclarationSpecifier::ImportDefaultSpecifier(d) => Some(d.local.name.as_str()),
            _ => None,
        })
    })
}

fn count_newlines(source: &str) -> usize {
    source.bytes().filter(|b| *b == b'\n').count()
}

/// Skips the fix when merging would drop or relocate comments, mirroring
/// `hasProblematicComments` in eslint-plugin-import. Checks for a comment on the line just before
/// the statement, a trailing comment on the statement's last line, or a comment outside the
/// specifier braces.
fn has_problematic_comments(ctx: &LintContext, decl: &ImportDeclaration) -> bool {
    let comments = ctx.semantic().comments();
    let span = decl.span;

    let comment_before = comments
        .iter()
        .filter(|c| c.span.end <= span.start)
        .max_by_key(|c| c.span.end)
        .is_some_and(|c| count_newlines(ctx.source_range(Span::new(c.span.end, span.start))) <= 1);
    if comment_before {
        return true;
    }

    let comment_after = comments
        .iter()
        .filter(|c| c.span.start >= span.end)
        .min_by_key(|c| c.span.start)
        .is_some_and(|c| count_newlines(ctx.source_range(Span::new(span.end, c.span.start))) == 0);
    if comment_after {
        return true;
    }

    let open_brace = brace_position(ctx, decl, "{");
    let close_brace = brace_position(ctx, decl, "}");
    comments.iter().filter(|c| c.span.start > span.start && c.span.end < span.end).any(|c| {
        match (open_brace, close_brace) {
            (Some(open), Some(close)) => c.span.end <= open || c.span.start >= close,
            _ => true,
        }
    })
}

fn brace_position(ctx: &LintContext, decl: &ImportDeclaration, brace: &str) -> Option<u32> {
    ctx.find_next_token_within(decl.span.start, decl.span.end, brace)
        .map(|offset| decl.span.start + offset)
}

struct MergeSpecifier<'a> {
    decl: &'a ImportDeclaration<'a>,
    identifiers: Vec<&'a str>,
    is_empty: bool,
    is_type: bool,
}

fn merge_imports_fix<'a>(
    fixer: RuleFixer<'_, 'a>,
    ctx: &LintContext<'a>,
    prefer_inline: bool,
    decls: &[&'a ImportDeclaration<'a>],
) -> RuleFix {
    let first = decls[0];
    let rest = &decls[1..];

    if decls.iter().any(|decl| {
        decl.source.value != first.source.value
            || decl.with_clause.is_some()
            || decl.phase.is_some()
    }) {
        return fixer.noop();
    }

    if has_namespace(first) || has_problematic_comments(ctx, first) {
        return fixer.noop();
    }

    let default_import_names: Vec<&str> =
        decls.iter().filter_map(|d| default_import_name(d)).unique().collect();
    if default_import_names.len() > 1 {
        return fixer.noop();
    }

    let mergeable: Vec<&'a ImportDeclaration<'a>> = rest
        .iter()
        .copied()
        .filter(|d| !has_namespace(d) && !has_problematic_comments(ctx, d))
        .collect();

    let specifiers: Vec<MergeSpecifier<'a>> = mergeable
        .iter()
        .copied()
        .filter_map(|decl| {
            let open = brace_position(ctx, decl, "{")?;
            let close = brace_position(ctx, decl, "}")?;
            let identifiers =
                ctx.source_range(Span::new(open + 1, close)).split(',').collect::<Vec<_>>();
            Some(MergeSpecifier {
                decl,
                identifiers,
                is_empty: !has_named_specifiers(decl),
                is_type: decl.import_kind.is_type(),
            })
        })
        .collect();

    let unnecessary: Vec<&'a ImportDeclaration<'a>> = mergeable
        .iter()
        .copied()
        .filter(|decl| {
            !has_named_specifiers(decl)
                && !has_namespace(decl)
                && !specifiers.iter().any(|s| std::ptr::eq(s.decl, *decl))
        })
        .collect();

    let should_add_default =
        default_import_name(first).is_none() && default_import_names.len() == 1;
    let should_add_specifiers = !specifiers.is_empty();
    let should_remove_unnecessary = !unnecessary.is_empty();

    if !(should_add_default || should_add_specifiers || should_remove_unnecessary) {
        return fixer.noop();
    }

    let open_brace = brace_position(ctx, first, "{");
    let close_brace = open_brace.and_then(|open| {
        ctx.find_next_token_within(open + 1, first.span.end, "}").map(|o| open + 1 + o)
    });
    let import_keyword_end = first.span.start + u32::try_from("import".len()).unwrap();

    let first_is_empty = !has_named_specifiers(first);
    let first_brace_content = match (open_brace, close_brace) {
        (Some(open), Some(close)) => ctx.source_range(Span::new(open + 1, close)),
        _ => "",
    };
    let first_has_trailing_comma = !first_is_empty && first_brace_content.trim_end().ends_with(',');

    let mut existing: FxHashSet<&str> = FxHashSet::default();
    if !first_is_empty {
        for id in first_brace_content.split(',') {
            existing.insert(id.trim());
        }
    }

    let specifiers_text = build_specifiers_text(
        &specifiers,
        &mut existing,
        first_has_trailing_comma,
        first_is_empty,
        prefer_inline,
    );

    let fixer = fixer.for_multifix();
    let mut fixes = fixer.new_fix_with_capacity(decls.len() + 1);

    if should_add_specifiers && prefer_inline && first.import_kind.is_type() {
        if let Some(offset) = ctx.find_next_token_from(first.span.start, "type") {
            let type_start = first.span.start + offset;
            let mut type_end = type_start + u32::try_from("type".len()).unwrap();
            if ctx.semantic().source_text().as_bytes().get(type_end as usize) == Some(&b' ') {
                type_end += 1;
            }
            fixes.push(fixer.delete_range(Span::new(type_start, type_end)));
        }
        if let Some(first_specifiers) = &first.specifiers {
            for specifier in first_specifiers {
                if matches!(specifier, ImportDeclarationSpecifier::ImportSpecifier(_)) {
                    fixes.push(fixer.insert_text_before(specifier, "type "));
                }
            }
        }
    }

    let default_name = default_import_names.first().copied().unwrap_or("");
    if should_add_default && open_brace.is_none() && should_add_specifiers {
        fixes.push(fixer.insert_text_before_range(
            Span::empty(import_keyword_end),
            format!(" {default_name}, {{{specifiers_text}}} from"),
        ));
    } else if should_add_default && open_brace.is_none() {
        fixes.push(fixer.insert_text_before_range(
            Span::empty(import_keyword_end),
            format!(" {default_name} from"),
        ));
    } else if should_add_default && let (Some(open), Some(close)) = (open_brace, close_brace) {
        fixes.push(fixer.insert_text_before_range(
            Span::empty(import_keyword_end),
            format!(" {default_name},"),
        ));
        if should_add_specifiers {
            fixes.push(merge_into_braces(&fixer, ctx, open, close, &specifiers_text));
        }
    } else if !should_add_default && open_brace.is_none() && should_add_specifiers {
        if first.specifiers.as_ref().is_none_or(|s| s.is_empty()) {
            fixes.push(fixer.insert_text_before_range(
                Span::empty(import_keyword_end),
                format!(" {{{specifiers_text}}} from"),
            ));
        } else if let Some(first_default) = first.specifiers.as_ref().and_then(|s| s.first()) {
            fixes.push(fixer.insert_text_after(first_default, format!(", {{{specifiers_text}}}")));
        }
    } else if !should_add_default && let (Some(open), Some(close)) = (open_brace, close_brace) {
        fixes.push(merge_into_braces(&fixer, ctx, open, close, &specifiers_text));
    }

    for specifier in &specifiers {
        push_removal(&mut fixes, &fixer, ctx, specifier.decl.span);
    }
    for decl in &unnecessary {
        push_removal(&mut fixes, &fixer, ctx, decl.span);
    }

    fixes.with_message("Merge duplicate imports into a single import statement")
}

fn build_specifiers_text<'a>(
    specifiers: &[MergeSpecifier<'a>],
    existing: &mut FxHashSet<&'a str>,
    first_has_trailing_comma: bool,
    first_is_empty: bool,
    prefer_inline: bool,
) -> String {
    let mut result = String::new();
    let mut needs_comma = !first_has_trailing_comma && !first_is_empty;

    for specifier in specifiers {
        let mut specifier_text = String::new();
        for cur in &specifier.identifiers {
            let trimmed = cur.trim();
            let with_type: Cow<str> = if !trimmed.is_empty() && prefer_inline && specifier.is_type {
                Cow::Owned(format!("type {trimmed}"))
            } else if !trimmed.is_empty() && ends_with_line_comment(trimmed) {
                Cow::Owned(format!("{trimmed}\n"))
            } else if cur.contains('\n') {
                Cow::Borrowed(strip_leading_inline_whitespace(cur.trim_end()))
            } else {
                Cow::Borrowed(trimmed)
            };
            if existing.contains(trimmed) {
                continue;
            }
            existing.insert(trimmed);
            if specifier_text.is_empty() {
                specifier_text.push_str(&with_type);
            } else {
                specifier_text.push(',');
                specifier_text.push_str(&with_type);
            }
        }

        if needs_comma && !specifier.is_empty && !specifier_text.is_empty() {
            result.push(',');
        }
        result.push_str(&specifier_text);
        if !specifier.is_empty {
            needs_comma = true;
        }
    }

    result
}

fn ends_with_line_comment(text: &str) -> bool {
    text.rsplit('\n').next().unwrap_or(text).contains("//")
}

fn strip_leading_inline_whitespace(text: &str) -> &str {
    text.trim_start_matches(|c: char| c.is_whitespace() && c != '\n')
}

fn merge_into_braces<'a>(
    fixer: &RuleFixer<'_, 'a>,
    ctx: &LintContext<'a>,
    open: u32,
    close: u32,
    specifiers_text: &str,
) -> RuleFix {
    let content = ctx.source_range(Span::new(open + 1, close));
    let trailing_whitespace_len = content.len() - content.trim_end().len();
    if content.trim().is_empty() {
        fixer.replace(Span::new(open + 1, close), specifiers_text.to_string())
    } else if trailing_whitespace_len > 0 {
        let trailing = &content[content.len() - trailing_whitespace_len..];
        fixer.replace(
            Span::new(close - u32::try_from(trailing_whitespace_len).unwrap(), close + 1),
            format!("{specifiers_text}{trailing}}}"),
        )
    } else {
        fixer.insert_text_before_range(Span::empty(close), specifiers_text.to_string())
    }
}

fn push_removal<'a>(
    fixes: &mut RuleFix,
    fixer: &RuleFixer<'_, 'a>,
    ctx: &LintContext<'a>,
    span: Span,
) {
    let mut end = span.end;
    if ctx.semantic().source_text().as_bytes().get(end as usize) == Some(&b'\n') {
        end += 1;
    }
    fixes.push(fixer.delete_range(Span::new(span.start, end)));
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (r#"import "./malformed.js""#, None),
        (r"import { x } from './foo'; import { y } from './bar'", None),
        (r#"import foo from "234artaf"; import { shoop } from "234q25ad""#, None),
        (r"import { x } from './foo'; import type { y } from './foo'", None),
        // #1107: Using different query strings that trigger different webpack loaders.
        // Test camelCase option
        (
            r"import x from './bar?optionX'; import y from './bar?optionY';",
            Some(json!([{ "considerQueryString": true }])),
        ),
        (
            r"import x from './foo'; import y from './bar';",
            Some(json!([{ "considerQueryString": true }])),
        ),
        // #1538: It is impossible to import namespace and other in one line, so allow this.
        (r"import * as ns from './foo'; import { y } from './foo'", None),
        (r"import { y } from './foo'; import * as ns from './foo'", None),
        // TypeScript
        (r"import type { x } from './foo'; import y from './foo'", None),
        (r"import type x from './foo'; import type y from './bar'", None),
        (r"import type {x} from './foo'; import type {y} from './bar'", None),
        (r"import type x from './foo'; import type {y} from './foo'", None),
        (
            r"import type {} from './module';
        import {} from './module2';",
            None,
        ),
        (
            r"import type { Identifier } from 'module';

        declare module 'module2' {
        import type { Identifier } from 'module';
        }

        declare module 'module3' {
        import type { Identifier } from 'module';
        }",
            None,
        ),
        (r"import { type x } from './foo'; import y from './foo'", None),
        (r"import { type x } from './foo'; import { y } from './foo'", None),
        (r"import { type x } from './foo'; import type y from 'foo'", None),
        (r"import { x } from './foo'; export { x } from './foo'", None),
        // for cases in https://github.com/import-js/eslint-plugin-import/issues/2750
        (r"import type * as something from './foo'; import type y from './foo';", None),
        (r"import type * as something from './foo'; import type { y } from './foo';", None),
        (r"import type y from './foo'; import type * as something from './foo';", None),
        (r"import type { y } from './foo'; import type * as something from './foo';", None),
        // type + import
        (r"import type * as something from './foo'; import y from './foo';", None),
        (r"import type * as something from './foo'; import { y } from './foo';", None),
        (r"import y from './foo'; import type * as something from './foo';", None),
        (r"import { y } from './foo'; import type * as something from './foo';", None),
        // A type-only import is not a duplicate of a bare side-effect import.
        (r"import type { Foo } from './foo'; import './foo';", None),
        (
            r"import type { Foo } from './foo'; import './foo';",
            Some(json!([{ "preferInline": true }])),
        ),
        (
            r"import './foo'; import type { Foo } from './foo';",
            Some(json!([{ "prefer-inline": true }])),
        ),
        (r"import type {} from './foo'; import './foo';", None),
        (r"import type {} from './foo'; import './foo';", Some(json!([{ "preferInline": true }]))),
        (r"import { RouterModule, Routes } from '@angular/router';", None),
    ];

    let fail = vec![
        (r"import { x } from './foo'; import { y } from './foo'", None),
        (r"import {x} from './foo'; import {y} from './foo'; import { z } from './foo'", None),
        // #1107: Using different query strings without considerQueryString (default false)
        // These should be flagged as duplicates because query strings are ignored by default
        (r"import x from './bar.js?optionX'; import y from './bar?optionX';", None),
        (r"import x from './bar?optionX'; import y from './bar?optionY';", None),
        (r"import x from './bar?optionX'; import y from './bar.js?optionX';", None),
        // #1107: Using same query strings with considerQueryString: true
        // Same file + same query string = duplicate
        (
            r"import x from './bar?optionX'; import y from './bar.js?optionX';",
            Some(json!([{ "considerQueryString": true }])),
        ),
        // #86: duplicate unresolved modules should be flagged
        (r"import foo from 'non-existent'; import bar from 'non-existent';", None),
        (r"import type { x } from './foo'; import type { y } from './foo'", None),
        (r"import './foo'; import './foo'", None),
        (
            r"import { x, /* x */ } from './foo'; import {//y
        y//y2
        } from './foo'",
            None,
        ),
        (r"import {x} from './foo'; import {} from './foo'", None),
        (r"import {a} from './foo'; import { a } from './foo'", None),
        (
            r"import {a,b} from './foo'; import { b, c } from './foo'; import {b,c,d} from './foo'",
            None,
        ),
        (r"import {a} from './foo'; import { a/*,b*/ } from './foo'", None),
        (r"import {a} from './foo'; import { a } from './foo'", None),
        (
            r"import {a,b} from './foo'; import { b, c } from './foo'; import {b,c,d} from './foo'",
            None,
        ),
        (r"import {a} from './foo'; import { a/*,b*/ } from './foo'", None),
        (
            r"import {x} from './foo'; import {} from './foo'; import {/*c*/} from './foo'; import {y} from './foo'",
            None,
        ),
        (r"import { } from './foo'; import {x} from './foo'", None),
        (r"import './foo'; import {x} from './foo'", None),
        (r"import'./foo'; import {x} from './foo'", None),
        (
            r"import './foo'; import { /*x*/} from './foo'; import {//y
        } from './foo'; import {z} from './foo'",
            None,
        ),
        (r"import './foo'; import def, {x} from './foo'", None),
        (r"import './foo'; import def from './foo'", None),
        (r"import def from './foo'; import {x} from './foo'", None),
        (r"import {x} from './foo'; import def from './foo'", None),
        (r"import{x} from './foo'; import def from './foo'", None),
        (r"import {x} from './foo'; import def, {y} from './foo'", None),
        (r"import * as ns1 from './foo'; import * as ns2 from './foo'", None),
        (r"import * as ns from './foo'; import {x} from './foo'; import {y} from './foo'", None),
        (
            r"import {x} from './foo'; import * as ns from './foo'; import {y} from './foo'; import './foo'",
            None,
        ),
        (
            r"// some-tool-disable-next-line
            import {x} from './foo'
            import {//y
        y} from './foo'",
            None,
        ),
        (
            r"import {x} from './foo'
            // some-tool-disable-next-line
            import {y} from './foo'",
            None,
        ),
        (
            r"import {x} from './foo' // some-tool-disable-line
            import {y} from './foo'",
            None,
        ),
        (
            r"import {x} from './foo'
            import {y} from './foo' // some-tool-disable-line",
            None,
        ),
        (
            r"import {x} from './foo'
            /* comment */ import {y} from './foo'",
            None,
        ),
        (
            r"import {x} from './foo'
            import {y} from './foo' /* comment
            multiline */",
            None,
        ),
        (
            r"import {x} from './foo'
        import {y} from './foo'
        // some-tool-disable-next-line",
            None,
        ),
        (
            r"import {x} from './foo'
        // comment

        import {y} from './foo'",
            None,
        ),
        (
            r"import {x} from './foo'
            import/* comment */{y} from './foo'",
            None,
        ),
        (
            r"import {x} from './foo'
            import/* comment */'./foo'",
            None,
        ),
        (
            r"import {x} from './foo'
            import{y}/* comment */from './foo'",
            None,
        ),
        (
            r"import {x} from './foo'
            import{y}from/* comment */'./foo'",
            None,
        ),
        (
            r"import {x} from
            // some-tool-disable-next-line
            './foo'
            import {y} from './foo'",
            None,
        ),
        (
            r"import { Foo } from './foo';
        import { Bar } from './foo';
        export const value = {}",
            None,
        ),
        (
            r"import { Foo } from './foo';
        import Bar from './foo';
        export const value = {}",
            None,
        ),
        (
            r"import {
              DEFAULT_FILTER_KEYS,
              BULK_DISABLED,
            } from '../constants';
            import React from 'react';
            import {
              BULK_ACTIONS_ENABLED
            } from '../constants';

            const TestComponent = () => {
              return <div>
              </div>;
            }

            export default TestComponent;",
            None,
        ),
        (
            r"import {A1,} from 'foo';
            import {B1,} from 'foo';
            import {C1,} from 'foo';

            import {
            A2,
            } from 'bar';
            import {
            B2,
            } from 'bar';
            import {
            C2,
            } from 'bar';",
            None,
        ),
        // TypeScript
        (r"import type x from './foo'; import type y from './foo'", None),
        (r"import type x from './foo'; import type x from './foo'", None),
        (r"import type {x} from './foo'; import type {y} from './foo'", None),
        // prefer-inline: false (default) - inline type and type imports are in same category
        (r"import {type x} from './foo'; import type {y} from './foo'", None),
        // prefer-inline: true - inline type and type imports are in same category
        (
            r"import {type x} from 'foo'; import type {y} from 'foo'",
            Some(json!([{ "prefer-inline": true }])),
        ),
        // prefer-inline: true - swapped order (type import first, then inline type)
        (
            r"import type {x} from 'foo'; import {type y} from 'foo'",
            Some(json!([{ "prefer-inline": true }])),
        ),
        // prefer-inline: false (default)
        (r"import {type x} from 'foo'; import type {y} from 'foo'", None),
        // prefer-inline: true - both inline type imports
        (
            r"import {type x} from './foo'; import {type y} from './foo'",
            Some(json!([{ "prefer-inline": true }])),
        ),
        // prefer-inline: false (default) - both inline type imports
        (r"import {type x} from './foo'; import {type y} from './foo'", None),
        (r"import {AValue, type x, BValue} from './foo'; import {type y} from './foo'", None),
        // #2834 Detect duplicates across type and regular imports with prefer-inline: true
        // Test prefer-inline with camelCase (legacy)
        (
            r"import {AValue} from './foo'; import type {AType} from './foo'",
            Some(json!([{ "preferInline": true }])),
        ),
        // Test prefer-inline with kebab-case (primary, matches ESLint)
        (
            r"import {AValue} from './foo'; import type {AType} from './foo'",
            Some(json!([{ "prefer-inline": true }])),
        ),
        // Genuine duplicates still reported (two type-only imports merge under prefer-inline).
        (
            r"import type {x} from './foo'; import type {y} from './foo'",
            Some(json!([{ "preferInline": true }])),
        ),
        // A side-effect import is still redundant when a value import also exists.
        (
            r"import './foo'; import { Bar } from './foo'; import type { Foo } from './foo'",
            Some(json!([{ "preferInline": true }])),
        ),
        (
            r"import './foo'; import type { Foo } from './foo'; import type { Bar } from './foo'",
            Some(json!([{ "preferInline": true }])),
        ),
        (
            r"import './foo'; import './foo'; import type { Foo } from './foo'",
            Some(json!([{ "preferInline": true }])),
        ),
        (
            r"import { type Foo } from './foo'; import './foo'",
            Some(json!([{ "preferInline": true }])),
        ),
    ];

    let fix = vec![
        (
            r"import { x } from './foo'; import { y } from './foo'",
            r"import { x,y } from './foo'; ",
            None,
        ),
        (
            r"import {x} from './foo'; import {y} from './foo'; import { z } from './foo'",
            r"import {x,y,z} from './foo';  ",
            None,
        ),
        (r"import './foo'; import {x} from './foo'", r"import {x} from './foo'; ", None),
        (
            r"import def from './foo'; import {x} from './foo'",
            r"import def, {x} from './foo'; ",
            None,
        ),
        (
            r"import {x} from './foo'; import def from './foo'",
            r"import def, {x} from './foo'; ",
            None,
        ),
        (r"import './foo'; import def, {x} from './foo'", r"import def, {x} from './foo'; ", None),
        (r"import './foo'; import def from './foo'", r"import def from './foo'; ", None),
        (r"import './foo'; import './foo'", r"import './foo'; ", None),
        (r"import {x} from './foo'; import {} from './foo'", r"import {x} from './foo'; ", None),
        (
            r"import type {x} from './foo'; import type {y} from './foo'",
            r"import type {x,y} from './foo'; ",
            None,
        ),
        (
            r"import {AValue} from './foo'; import type {AType} from './foo'",
            r"import {AValue,type AType} from './foo'; ",
            Some(json!([{ "prefer-inline": true }])),
        ),
        (
            r"import type {x} from 'foo'; import {type y} from 'foo'",
            r"import {type x,type y} from 'foo'; ",
            Some(json!([{ "prefer-inline": true }])),
        ),
        // Namespace as the first import cannot be merged.
        (
            r"import * as ns1 from './foo'; import * as ns2 from './foo'",
            r"import * as ns1 from './foo'; import * as ns2 from './foo'",
            None,
        ),
        // Conflicting default names cannot be merged.
        (
            r"import type x from './foo'; import type y from './foo'",
            r"import type x from './foo'; import type y from './foo'",
            None,
        ),
        // A problematic comment on the first import blocks the fix.
        (
            r"import {x} from './foo' // some-tool-disable-line
            import {y} from './foo'",
            r"import {x} from './foo' // some-tool-disable-line
            import {y} from './foo'",
            None,
        ),
        // Merging multiple named specifiers from both imports.
        (
            r"import {a,b} from 'foo'; import {c,d} from 'foo'",
            r"import {a,b,c,d} from 'foo'; ",
            None,
        ),
        // A top-level type import merges into a value import as inline type specifiers.
        (
            r"import type {a,b} from 'foo'; import {c,d} from 'foo'",
            r"import {type a,type b,c,d} from 'foo'; ",
            Some(json!([{ "prefer-inline": true }])),
        ),
        // A trailing comma in the first import is preserved when appending specifiers.
        (r"import {a,} from 'foo'; import {b} from 'foo'", r"import {a,b} from 'foo'; ", None),
        // Imports with different query strings must not be merged, even when the rule considers
        // them duplicates for reporting purposes.
        (
            r"import {x} from './foo?one'; import {y} from './foo?two'",
            r"import {x} from './foo?one'; import {y} from './foo?two'",
            None,
        ),
        // Import attributes can change module semantics and must not be discarded by a merge.
        (
            r"import {x} from './foo' with { type: 'json' }; import {y} from './foo' with { type: 'css' }",
            r"import {x} from './foo' with { type: 'json' }; import {y} from './foo' with { type: 'css' }",
            None,
        ),
    ];

    Tester::new(NoDuplicates::NAME, NoDuplicates::PLUGIN, pass, fail)
        .change_rule_path("index.ts")
        .with_import_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
