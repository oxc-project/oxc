use std::ops::Deref;

use itertools::Itertools;
use oxc_ast::{
    ast::{
        ImportDeclaration, ImportDeclarationSpecifier, ImportDefaultSpecifier,
        ImportNamespaceSpecifier, ImportSpecifier,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{CompactStr, GetSpan, Span, SPAN};

use crate::{
    context::LintContext,
    fixer::{Fix, RuleFixer},
    rule::Rule,
    AstNode,
};

fn no_import_type_annotations_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "typescript-eslint(consistent-type-imports): `import()` type annotations are forbidden.",
    )
    .with_label(span)
}

fn avoid_import_type_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "typescript-eslint(consistent-type-imports): Use an `import` instead of an `import type`.",
    )
    .with_label(span)
}
fn type_over_value_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn( "typescript-eslint(consistent-type-imports): All imports in the declaration are only used as types. Use `import type`."
    )
    .with_label(span)
}

fn some_imports_are_only_types_diagnostic(span0: Span, x1: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "typescript-eslint(consistent-type-imports): Imports {x1} are only used as type."
    ))
    .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentTypeImports(Box<ConsistentTypeImportsConfig>);

impl Deref for ConsistentTypeImports {
    type Target = ConsistentTypeImportsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// <https://github.com/typescript-eslint/typescript-eslint/blob/main/packages/eslint-plugin/docs/rules/consistent-type-imports.mdx>
#[derive(Default, Debug, Clone)]
pub struct ConsistentTypeImportsConfig {
    disallow_type_annotations: DisallowTypeAnnotations,
    fix_style: FixStyle,
    prefer: Prefer,
}

// The default of `disallowTypeAnnotations` is `true`.
#[derive(Debug, Clone)]
struct DisallowTypeAnnotations(bool);

impl DisallowTypeAnnotations {
    fn new(value: bool) -> Self {
        Self(value)
    }
}

impl Default for DisallowTypeAnnotations {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Default, Debug, Clone, Copy)]
enum FixStyle {
    #[default]
    SeparateTypeImports,
    InlineTypeImports,
}

#[derive(Default, Debug, Clone)]
enum Prefer {
    #[default]
    TypeImports,
    NoTypeImports,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce consistent usage of type imports.
    ///
    /// ### Why is this bad?
    ///
    /// inconsistent usage of type imports can make the code harder to read and understand.
    ///
    /// ### Example
    /// ```javascript
    /// import { Foo } from 'Foo';
    /// type T = Foo;
    ///
    /// type S = import("Foo");
    /// ```
    ConsistentTypeImports,
    nursery,
);

impl Rule for ConsistentTypeImports {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0).and_then(serde_json::Value::as_object).map_or(
            ConsistentTypeImportsConfig::default(),
            |config| {
                let disallow_type_annotations = config
                    .get("disallowTypeAnnotations")
                    .and_then(serde_json::Value::as_bool)
                    .map(DisallowTypeAnnotations::new)
                    .unwrap_or_default();
                let fix_style = config.get("fixStyle").and_then(serde_json::Value::as_str).map_or(
                    FixStyle::SeparateTypeImports,
                    |fix_style| match fix_style {
                        "inline-type-imports" => FixStyle::InlineTypeImports,
                        _ => FixStyle::SeparateTypeImports,
                    },
                );
                let prefer = config.get("prefer").and_then(serde_json::Value::as_str).map_or(
                    Prefer::TypeImports,
                    |prefer| match prefer {
                        "no-type-imports" => Prefer::NoTypeImports,
                        _ => Prefer::TypeImports,
                    },
                );

                ConsistentTypeImportsConfig { disallow_type_annotations, fix_style, prefer }
            },
        );
        Self(Box::new(config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if self.disallow_type_annotations.0 {
            //  `import()` type annotations are forbidden.
            // `type Foo = import('foo')`
            if let AstKind::TSImportType(import_type) = node.kind() {
                ctx.diagnostic(no_import_type_annotations_diagnostic(import_type.span));
                return;
            }
        }

        if matches!(self.prefer, Prefer::NoTypeImports) {
            match node.kind() {
                // `import type { Foo } from 'foo'`
                AstKind::ImportDeclaration(import_decl) => {
                    if import_decl.import_kind.is_type() {
                        ctx.diagnostic(avoid_import_type_diagnostic(import_decl.span));
                    }
                }
                // import { type Foo } from 'foo'
                AstKind::ImportSpecifier(import_specifier) => {
                    if import_specifier.import_kind.is_type() {
                        ctx.diagnostic(avoid_import_type_diagnostic(import_specifier.span));
                    }
                }
                _ => {}
            }
            return;
        }

        let AstKind::ImportDeclaration(import_decl) = node.kind() else {
            return;
        };

        // Store references that only used as type and without type qualifier.
        // For example:
        // ```typescript
        // import { A, B, type C } from 'foo';
        // const a: A;
        // const b: B;
        // const c: C;
        // ```
        // `A` and `B` are only used as type references.
        let mut type_references_without_type_qualifier = vec![];
        // If all specifiers are only used as type references.
        let mut is_only_type_references = false;

        if let Some(specifiers) = &import_decl.specifiers {
            for specifier in specifiers {
                let Some(symbol_id) = specifier.local().symbol_id.get() else {
                    continue;
                };
                let no_type_qualifier = match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        specifier.import_kind.is_value()
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(_)
                    | ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => true,
                };

                if no_type_qualifier && is_only_has_type_references(symbol_id, ctx) {
                    type_references_without_type_qualifier.push(specifier);
                }
            }

            is_only_type_references =
                type_references_without_type_qualifier.len() == specifiers.len();
        }

        if import_decl.import_kind.is_value() && !type_references_without_type_qualifier.is_empty()
        {
            // `import type {} from 'foo' assert { type: 'json' }` is invalid
            // Import assertions cannot be used with type-only imports or exports.
            if is_only_type_references && import_decl.with_clause.is_none() {
                ctx.diagnostic(type_over_value_diagnostic(import_decl.span));
                return;
            }

            let type_names = type_references_without_type_qualifier
                .iter()
                .map(|specifier| specifier.name())
                .collect::<Vec<_>>();

            // ['foo', 'bar', 'baz' ] => "foo, bar, and baz".
            let type_imports = format_word_list(&type_names);

            ctx.diagnostic_with_fix(
                some_imports_are_only_types_diagnostic(import_decl.span, &type_imports),
                |fixer| {
                    let type_names = type_names.iter().map(CompactStr::as_str).collect::<Vec<_>>();
                    let fix_options = FixOptions {
                        fixer,
                        import_decl,
                        type_names: &type_names,
                        fix_style: self.fix_style,
                        ctx,
                    };

                    fix_to_type_import_declaration(&fix_options)
                },
            );
        }
    }
}

// Given an array of words, returns an English-friendly concatenation, separated with commas, with
// the `and` clause inserted before the last item.
//
// Example: ['foo', 'bar', 'baz' ] returns the string "foo, bar, and baz".
fn format_word_list(words: &[CompactStr]) -> String {
    match words.len() {
        0 => String::new(),
        1 => words[0].to_string(),
        2 => format!("{} and {}", words[0], words[1]),
        _ => {
            let mut result = String::new();
            for (i, word) in words.iter().enumerate() {
                if i == words.len() - 1 {
                    result.push_str(&format!("and {word}"));
                } else {
                    result.push_str(&format!("{word}, "));
                }
            }
            result
        }
    }
}

// Returns `true` if the symbol is only used as a type reference, and `false` otherwise.
// Specifically, return `false` if the symbol does not have any references.
fn is_only_has_type_references(symbol_id: SymbolId, ctx: &LintContext) -> bool {
    let mut peekable_iter = ctx.semantic().symbol_references(symbol_id).peekable();

    if peekable_iter.peek().is_none() {
        return false;
    }
    peekable_iter.all(oxc_semantic::Reference::is_type)
}

struct FixOptions<'a, 'b> {
    fixer: RuleFixer<'b, 'a>,
    import_decl: &'b ImportDeclaration<'a>,
    type_names: &'b [&'b str],
    fix_style: FixStyle,
    ctx: &'b LintContext<'a>,
}

// import { Foo, Bar } from 'foo' => import type { Foo, Bar } from 'foo'
#[allow(clippy::unnecessary_cast, clippy::cast_possible_truncation)]
fn fix_to_type_import_declaration<'a>(options: &FixOptions<'a, '_>) -> Vec<Fix<'a>> {
    let FixOptions { fixer, import_decl, type_names, fix_style, ctx } = options;

    let GroupedSpecifiers { namespace_specifier, named_specifiers, default_specifier } =
        classify_specifier(import_decl);

    // import * as type from 'foo'
    if namespace_specifier.is_some()
        && default_specifier.is_none()
        // checks for presence of import assertions
        && import_decl.with_clause.is_none()
    {
        return fix_insert_type_specifier_for_import_declaration(
            options, /* is_default_import */ false,
        );
    } else if let Some(default_specifier) = default_specifier {
        // import Type from 'foo'
        if type_names.iter().contains(&default_specifier.local.name.as_str())
            && named_specifiers.is_empty()
            && namespace_specifier.is_none()
        {
            return fix_insert_type_specifier_for_import_declaration(
                options, /* is_default_import */ true,
            );
        } else if matches!(fix_style, FixStyle::InlineTypeImports)
            && !type_names.iter().contains(&default_specifier.local.name.as_str())
            && !named_specifiers.is_empty()
            && namespace_specifier.is_some()
        {
            // if there is a default specifier but it isn't a type specifier, then just add the inline type modifier to the named specifiers
            // import AValue, {BValue, Type1, Type2} from 'foo'
            return fix_inline_type_import_declaration(options);
        } else if named_specifiers
            .iter()
            .all(|specifier| type_names.iter().contains(&specifier.local.name.as_str()))
        {
            // import { Type1, Type2 } from 'foo'
            return fix_insert_type_specifier_for_import_declaration(
                options, /* is_default_import */ false,
            );
        }
    }

    let type_names_specifiers = named_specifiers
        .iter()
        .filter(|specifier| type_names.iter().contains(&specifier.local.name.as_str()))
        .copied()
        .collect::<Vec<_>>();

    let fixes_named_specifiers = get_fixes_named_specifiers(options, &type_names_specifiers);
    let mut fixes = vec![];

    if type_names_specifiers.is_empty() {
        // The import is both default and named.  Insert named on new line because can't mix default type import and named type imports
        if matches!(fix_style, FixStyle::InlineTypeImports) {
            let text = format!(
                "import {} from {}",
                type_names_specifiers
                    .iter()
                    .map(|spec| {
                        let insert_text = ctx.source_range(spec.span());
                        format!("type {insert_text}")
                    })
                    .join(", "),
                import_decl.source.value,
            );
            fixes.push(fixer.insert_text_before(*import_decl, text));
        } else {
            fixes.push(fixer.insert_text_before(
                *import_decl,
                format!(
                    "import type {{ {} }} from {}",
                    fixes_named_specifiers.type_named_specifiers_text, import_decl.source.value,
                ),
            ));
        }
    } else {
        // `type_names` is all type references, if `type_names_specifiers.len() ==
        // type_names.len()`, it means all type references are name specifiers.
        let type_only_named_import = type_names_specifiers.len() == type_names.len();
        if type_only_named_import {
            fixes.push(fix_insert_named_specifiers_in_named_specifier_list(
                options,
                &fixes_named_specifiers.type_named_specifiers_text,
            ));
        }
    }

    let mut fixes_remove_type_namespace_specifier = vec![];

    if let Some(namespace_specifier) = namespace_specifier {
        if type_names.iter().contains(&namespace_specifier.local.name.as_str()) {
            // import Foo, * as Type from 'foo'
            // import DefType, * as Type from 'foo'
            // import DefType, * as Type from 'foo'
            let comma = find_char_with_null_assert(ctx.source_range(import_decl.span), ',');

            // import Def, * as Ns from 'foo'
            //           ^^^^^^^^^ remove
            fixes_remove_type_namespace_specifier.push(fixer.delete(&Span::new(
                import_decl.span.start + comma,
                namespace_specifier.span().end,
            )));

            // import type * as Ns from 'foo'
            // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ insert
            fixes.push(fixer.insert_text_before(
                *import_decl,
                format!(
                    "import type {} from {}",
                    ctx.source_range(namespace_specifier.span()),
                    import_decl.source.value
                ),
            ));
        }
    }

    if let Some(default_specifier) = default_specifier {
        if type_names.iter().contains(&default_specifier.local.name.as_str()) {
            if type_names.len() == import_decl.specifiers.as_ref().map_or(0, |s| s.len()) {
                // import type Type from 'foo'
                //        ^^^^^ insert
                fixes.push(fixer.insert_text_after(
                    &Span::new(import_decl.span().start, import_decl.span().start + 6),
                    " type",
                ));
            } else {

                // import Type, { Foo } from 'foo'
            }
        } else {
            let import_text = ctx.source_range(import_decl.span);
            let comma = find_char_with_null_assert(import_text, ',');
            // import Type , { ... } from 'foo'
            //        ^^^^^ pick
            let default_text = ctx.source_range(default_specifier.span);
            fixes.push(fixer.insert_text_before(
                *import_decl,
                format!("import type {default_text} from {}\n", import_decl.source.value),
            ));
            // find the first non-whitespace character after the comma
            let mut after_token = comma + 1;
            for ch in import_text[(comma + 1) as usize..].chars() {
                if !ch.is_whitespace() {
                    break;
                }
                after_token += ch.len_utf8() as u32;
            }
            fixes.push(
                fixer.delete_range(Span::new(default_specifier.span.start, after_token as u32)),
            );
        }

        fixes.extend(fixes_named_specifiers.reomve_type_name_specifiers);
        fixes.extend(fixes_remove_type_namespace_specifier);
    }

    fixes
}

fn fix_insert_named_specifiers_in_named_specifier_list<'a>(
    options: &FixOptions<'a, '_>,
    insert_text: &str,
) -> Fix<'a> {
    let FixOptions { fixer, import_decl, ctx, .. } = options;
    let import_text = ctx.source_range(import_decl.span);
    let close_brace = find_char_with_null_assert(import_text, '}');

    let first_non_whitespace_before_close_brace =
        import_text[..close_brace as usize].chars().rev().find(|c| c.is_whitespace());

    let span =
        Span::new(import_decl.span().start + close_brace, import_decl.span().start + close_brace);
    if first_non_whitespace_before_close_brace.is_some_and(|ch| matches!(ch, ',' | '{')) {
        fixer.insert_text_before(&span, format!(",{insert_text}"))
    } else {
        fixer.insert_text_before(&span, insert_text.to_string())
    }
}

// Returns information for fixing named specifiers, type or value
#[derive(Default)]
struct FixNamedSpecifiers<'a> {
    type_named_specifiers_text: String,
    reomve_type_name_specifiers: Vec<Fix<'a>>,
}
fn get_fixes_named_specifiers<'a>(
    options: &FixOptions<'a, '_>,
    subset_named_specifiers: &[&ImportSpecifier<'a>],
) -> FixNamedSpecifiers<'a> {
    let FixOptions { fixer, import_decl, ctx, .. } = options;

    let Some(specifiers) = &import_decl.specifiers else {
        return FixNamedSpecifiers::default();
    };

    let mut type_named_specifiers_text: Vec<&str> = vec![];
    let mut remove_type_named_specifiers: Vec<Fix> = vec![];

    if subset_named_specifiers.len() == specifiers.len() {
        // import Foo, {Type1, Type2} from 'foo'
        // import DefType, {Type1, Type2} from 'foo'
        let import_text = ctx.source_range(import_decl.span);
        let Some(open_brace_token) = import_text.find('{') else {
            debug_assert!(false, "Missing open brace token in import declaration: {import_text}");
            return FixNamedSpecifiers::default();
        };
        let Some(comma_token) = import_text[..open_brace_token].rfind(',') else {
            debug_assert!(false, "Missing comma token in import declaration: {import_text}");
            return FixNamedSpecifiers::default();
        };
        let Some(close_brace_token) = import_text.find('}') else {
            debug_assert!(false, "Missing close brace token in import declaration: {import_text}");
            return FixNamedSpecifiers::default();
        };
        let open_brace_token_end = open_brace_token + 1;
        let close_brace_token_end = close_brace_token + 1;

        // import DefType, {...} from 'foo'
        //                ^^^^^^ remove
        remove_type_named_specifiers.push(fixer.delete(&Span::new(
            import_decl.span.start + u32::try_from(comma_token).unwrap_or_default(),
            import_decl.span.start + u32::try_from(close_brace_token_end).unwrap_or_default(),
        )));

        type_named_specifiers_text.push(&import_text[open_brace_token_end..close_brace_token]);
    } else {
        let mut named_specifier_groups = vec![];
        let mut group = vec![];

        for specifier in specifiers {
            let name = specifier.name();
            if subset_named_specifiers.iter().any(|s| s.local.name == name) {
                group.push(specifier);
            } else if !group.is_empty() {
                named_specifier_groups.push(group);
                group = vec![];
            }
        }

        if !group.is_empty() {
            named_specifier_groups.push(group);
        }

        for named_specifiers in named_specifier_groups {
            let (remove_range, text_range) =
                get_named_specifier_ranges(&named_specifiers, specifiers, options);

            remove_type_named_specifiers.push(fixer.delete(&remove_range));
            type_named_specifiers_text.push(ctx.source_range(text_range));
        }
    }

    FixNamedSpecifiers {
        type_named_specifiers_text: type_named_specifiers_text.join(","),
        reomve_type_name_specifiers: remove_type_named_specifiers,
    }
}

fn get_named_specifier_ranges(
    named_specifier_group: &[&ImportDeclarationSpecifier],
    all_specifiers: &[ImportDeclarationSpecifier],
    options: &FixOptions<'_, '_>,
) -> (/* remove_range */ Span, /* text_range*/ Span) {
    let FixOptions { ctx, import_decl, .. } = options;

    // It will never empty, in `get_fixes_named_specifiers`, we have already checked every group is
    // not empty.
    //
    if named_specifier_group.is_empty() {
        return (SPAN, SPAN);
    }
    let first = named_specifier_group[0];
    let last = named_specifier_group[named_specifier_group.len() - 1];

    let mut remove_range = Span::new(first.span().start, last.span().end);
    let mut text_range = Span::new(first.span().start, last.span().end);

    let import_text = ctx.source_range(import_decl.span);
    // This never fails as we are looking name specifers.
    let Some(open_brace_token_start) = import_text.find('{') else {
        debug_assert!(false, "Missing open brace token in import declaration: {import_text}");
        return (SPAN, SPAN);
    };
    if let Some(comma) = import_text[open_brace_token_start..first.span().start as usize].rfind(',')
    {
        // It's not the first specifier.
        // import { Foo, Bar, Baz } from 'foo'
        //             ^ start
        remove_range.start = import_decl.span().start
            + u32::try_from(comma + open_brace_token_start).unwrap_or_default();

        // Skip the comma
        text_range.start = remove_range.start + 1;
    } else {
        // It's the first specifier.
        // import { Foo, Bar, Baz } from 'foo'
        //         ^ start
        remove_range.start =
            first.span().start + u32::try_from(open_brace_token_start + 1).unwrap_or_default();
        text_range.start = remove_range.start;
    }

    let is_last =
        all_specifiers.last().is_some_and(|last_specifier| last_specifier.span() == last.span());

    if is_last {
        let after = find_char_with_null_assert(import_text, '}');
        // import { Foo, Bar, Baz } from 'foo'
        //                        ^ end
        text_range.end = import_decl.span().start + after;
    } else {
        let after = find_char_with_null_assert(&import_text[last.span().end as usize..], ',');
        // import { Foo, Bar, Baz } from 'foo'
        //                  ^ end
        text_range.end = import_decl.span().start + last.span().end + after;
        // import { Foo, Bar, Baz } from 'foo'
        //                   ^ end
        remove_range.end = text_range.end + 1;
    }

    (remove_range, text_range)
}

// Find the index of the first occurrence of a character in a string.
// When call this method, **make sure the `c` is in the text.**
// e.g. I know "{" is in the text when call this in ImportDeclaration which contains named
// specifiers.
fn find_char_with_null_assert(text: &str, c: char) -> u32 {
    let index = text.find(c);
    if let Some(index) = index {
        u32::try_from(index).unwrap_or_default()
    } else {
        debug_assert!(false, "Missing char: {c} in {text}");
        0
    }
}

fn fix_inline_type_import_declaration<'a>(options: &FixOptions<'a, '_>) -> Vec<Fix<'a>> {
    let FixOptions { fixer, import_decl, type_names, ctx, .. } = options;

    let mut fixes = vec![];

    let Some(specifiers) = &import_decl.specifiers else {
        return fixes;
    };

    for specifier in specifiers {
        if let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier {
            if type_names.iter().contains(&specifier.local.name.as_str()) {
                fixes.push(
                    fixer.replace(
                        specifier.span,
                        format!("type {}", ctx.source_range(specifier.span)),
                    ),
                );
            }
        }
    }

    fixes
}

fn fix_insert_type_specifier_for_import_declaration<'a>(
    options: &FixOptions<'a, '_>,
    is_default_import: bool,
) -> Vec<Fix<'a>> {
    let FixOptions { fixer, import_decl, ctx, .. } = options;
    let import_source = ctx.source_range(import_decl.span);
    let mut fixes = vec![];

    // "import { Foo, Bar } from 'foo'" => "import type { Foo, Bar } from 'foo'"
    //                                             ^^^^ add
    fixes.push(
        fixer.replace(Span::new(import_decl.span.start, import_decl.span.start + 5), "import type"),
    );

    if is_default_import {
        if let Some(_opening_brace_token) = import_source.find('{') {
            // `import foo, {} from 'foo'`
            // `import foo, { bar } from 'foo'`
            let Some(comma_token) = import_source.find(',') else {
                debug_assert!(false, "Missing comma token in import declaration");
                return vec![];
            };
            let Some(closing_brace_token) = import_source.find('}') else {
                debug_assert!(false, "Missing closing brace token in import declaration");
                return vec![];
            };
            let base = import_decl.span.start;
            // import foo, {} from 'foo'
            //           ^^^^ delete
            fixes.push(fixer.delete(&Span::new(
                base + u32::try_from(comma_token).unwrap_or_default(),
                base + u32::try_from(closing_brace_token + 1).unwrap_or_default(),
            )));
            if import_decl.specifiers.as_ref().is_some_and(|specifiers| !specifiers.is_empty()) {
                let Some(specifiers_text) =
                    import_source.get((comma_token + 1)..closing_brace_token)
                else {
                    debug_assert!(
                        false,
                        "Invalid slice for {}[{}..{}]",
                        import_source,
                        comma_token + 1,
                        closing_brace_token
                    );
                    return vec![];
                };

                fixes.push(fixer.insert_text_after(
                    *import_decl,
                    format!("\nimport type {} from {}", specifiers_text, import_decl.source.value),
                ));
            }
        }
    }

    if let Some(specifiers) = &import_decl.specifiers {
        for specifier in specifiers {
            if let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier {
                if specifier.import_kind.is_type() {
                    // import { type    A } from 'foo.js'
                    //          ^^^^^^^^ delete
                    fixes.push(
                        fixer.delete(&Span::new(
                            specifier.span.start,
                            specifier.imported.span().start,
                        )),
                    );
                }
            }
        }
    }

    fixes
}

struct GroupedSpecifiers<'a, 'b> {
    namespace_specifier: Option<&'b ImportNamespaceSpecifier<'a>>,
    named_specifiers: Vec<&'b ImportSpecifier<'a>>,
    default_specifier: Option<&'b ImportDefaultSpecifier<'a>>,
}

fn classify_specifier<'a, 'b>(import_decl: &'b ImportDeclaration<'a>) -> GroupedSpecifiers<'a, 'b> {
    let mut namespace_specifier = None;
    let mut named_specifiers = vec![];
    let mut default_specifier = None;

    let Some(specifiers) = &import_decl.specifiers else {
        return GroupedSpecifiers { namespace_specifier, named_specifiers, default_specifier };
    };

    for specifier in specifiers {
        match specifier {
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace) => {
                namespace_specifier = Some(namespace);
            }
            ImportDeclarationSpecifier::ImportSpecifier(named) => {
                named_specifiers.push(named);
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                default_specifier = Some(default);
            }
        }
    }

    GroupedSpecifiers { namespace_specifier, named_specifiers, default_specifier }
}

// import type Foo from 'foo'
//        ^^^^ remove
fn fix_remove_type_specifier_from_import_declaration<'a>(
    fixer: RuleFixer<'_, 'a>,
    import_decl_span: Span,
    ctx: &LintContext<'a>,
) -> Fix<'a> {
    let import_source = ctx.source_range(import_decl_span);
    let new_import_source = import_source
        // `    type Foo from 'foo'`
        .strip_prefix("import")
        // `type Foo from 'foo'`
        .map(str::trim_start)
        // `type Foo from 'foo'`
        .and_then(|import_text| import_text.strip_prefix("type"))
        // `import Foo from 'foo'`
        .map(|import_text| format!("import {import_text}"));

    if let Some(new_import_source) = new_import_source {
        fixer.replace(import_decl_span, new_import_source)
    } else {
        // when encountering an unexpected import declaration, do nothing.
        fixer.replace(import_decl_span, import_source)
    }
}

// import { type Foo } from 'foo'
//          ^^^^ remove
fn fix_remove_type_specifier_from_import_specifier<'a>(
    fixer: RuleFixer<'_, 'a>,
    specifier_span: Span,
    ctx: &LintContext<'a>,
) -> Fix<'a> {
    let specifier_source = ctx.source_range(specifier_span);
    let new_specifier_source = specifier_source.strip_prefix("type");

    fixer.replace(specifier_span, new_specifier_source.unwrap_or(specifier_source))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
              import Foo from 'foo';
              const foo: Foo = new Foo();
            ",
            None,
        ),
        (
            "
              import foo from 'foo';
              const foo: foo.Foo = foo.fn();
            ",
            None,
        ),
        (
            "
              import { A, B } from 'foo';
              const foo: A = B();
              const bar = new A();
            ",
            None,
        ),
        (
            "
              import Foo from 'foo';
                  ",
            None,
        ),
        // TODO: Need fix: https://github.com/oxc-project/oxc/issues/3799
        // (
        //     "
        //       import Foo from 'foo';
        //       type T<Foo> = Foo; // shadowing
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       import Foo from 'foo';
        //       function fn() {
        //         type Foo = {}; // shadowing
        //         let foo: Foo;
        //       }
        //     ",
        //     None,
        // ),
        (
            "
              import { A, B } from 'foo';
              const b = B;
            ",
            None,
        ),
        (
            "
              import { A, B, C as c } from 'foo';
              const d = c;
            ",
            None,
        ),
        (
            "
              import {} from 'foo'; // empty
            ",
            None,
        ),
        (
            "
              let foo: import('foo');
              let bar: import('foo').Bar;
            ",
            Some(serde_json::json!([{ "disallowTypeAnnotations": false }])),
        ),
        (
            "
              import Foo from 'foo';
              let foo: Foo;
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        // (
        //     "
        //       import type Type from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       import type { Type } from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       import type * as Type from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       import Type from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        // ),
        // (
        //     "
        //       import { Type } from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        // ),
        // (
        //     "
        //       import * as Type from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        // ),
        (
            "
              import * as Type from 'foo' assert { type: 'json' };
              const a: typeof Type = Type;
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            "
              import { type A } from 'foo';
              type T = A;
            ",
            None,
        ),
        (
            "
              import { type A, B } from 'foo';
              type T = A;
              const b = B;
            ",
            None,
        ),
        (
            "
              import { type A, type B } from 'foo';
              type T = A;
              type Z = B;
            ",
            None,
        ),
        (
            "
              import { B } from 'foo';
              import { type A } from 'foo';
              type T = A;
              const b = B;
            ",
            None,
        ),
        (
            "
              import { B, type A } from 'foo';
              type T = A;
              const b = B;
            ",
            Some(serde_json::json!([{ "fixStyle": "inline-type-imports" }])),
        ),
        (
            "
              import { B } from 'foo';
              import type A from 'baz';
              type T = A;
              const b = B;
            ",
            Some(serde_json::json!([{ "fixStyle": "inline-type-imports" }])),
        ),
        (
            "
              import { type B } from 'foo';
              import type { A } from 'foo';
              type T = A;
              const b = B;
            ",
            Some(serde_json::json!([{ "fixStyle": "inline-type-imports" }])),
        ),
        (
            "
              import { B, type C } from 'foo';
              import type A from 'baz';
              type T = A;
              type Z = C;
              const b = B;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { B } from 'foo';
              import type { A } from 'foo';
              type T = A;
              const b = B;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { B } from 'foo';
              import { A } from 'foo';
              type T = A;
              const b = B;
            ",
            Some(
                serde_json::json!([            { "prefer": "no-type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import Type from 'foo';
              
              export { Type }; // is a value export
              export default Type; // is a value export
            ",
            None,
        ),
        (
            "
              import type Type from 'foo';
        
              export { Type }; // is a type-only export
              export default Type; // is a type-only export
              export type { Type }; // is a type-only export
            ",
            None,
        ),
        (
            "
              import { Type } from 'foo';
        
              export { Type }; // is a value export
              export default Type; // is a value export
            ",
            None,
        ),
        (
            "
              import type { Type } from 'foo';
        
              export { Type }; // is a type-only export
              export default Type; // is a type-only export
              export type { Type }; // is a type-only export
            ",
            None,
        ),
        (
            "
              import * as Type from 'foo';
        
              export { Type }; // is a value export
              export default Type; // is a value export
            ",
            None,
        ),
        (
            "
              import type * as Type from 'foo';
        
              export { Type }; // is a type-only export
              export default Type; // is a type-only export
              export type { Type }; // is a type-only export
            ",
            None,
        ),
        (
            "
              import Type from 'foo';
        
              export { Type }; // is a type-only export
              export default Type; // is a type-only export
              export type { Type }; // is a type-only export
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            "
              import { Type } from 'foo';
        
              export { Type }; // is a type-only export
              export default Type; // is a type-only export
              export type { Type }; // is a type-only export
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            "
              import * as Type from 'foo';
        
              export { Type }; // is a type-only export
              export default Type; // is a type-only export
              export type { Type }; // is a type-only export
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        // TODO: https://github.com/typescript-eslint/typescript-eslint/issues/2455#issuecomment-685015542
        // import React has side effect.
        // (
        //     "
        //       import React from 'react';

        //       export const ComponentFoo: React.FC = () => {
        //         return <div>Foo Foo</div>;
        //       };
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       import { h } from 'some-other-jsx-lib';

        //       export const ComponentFoo: h.FC = () => {
        //         return <div>Foo Foo</div>;
        //       };
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       import { Fragment } from 'react';

        //       export const ComponentFoo: Fragment = () => {
        //         return <>Foo Foo</>;
        //       };
        //     ",
        //     None,
        // ),
        (
            "
              import Default, * as Rest from 'module';
              const a: typeof Default = Default;
              const b: typeof Rest = Rest;
            ",
            None,
        ),
        (
            "
              import type * as constants from './constants';
        
              export type Y = {
                [constants.X]: ReadonlyArray<string>;
              };
            ",
            None,
        ),
        (
            "
              import A from 'foo';
              export = A;
            ",
            None,
        ),
        (
            "
              import type A from 'foo';
              export = A;
            ",
            None,
        ),
        (
            "
              import type A from 'foo';
              export = {} as A;
            ",
            None,
        ),
        (
            "
              import { type A } from 'foo';
              export = {} as A;
            ",
            None,
        ),
        (
            "
              import type T from 'mod';
              const x = T;
            ",
            None,
        ),
        (
            "
              import type { T } from 'mod';
              const x = T;
            ",
            None,
        ),
        (
            "
              import { type T } from 'mod';
              const x = T;
            ",
            None,
        ),
        // TODO: To support decorator in this rule, need <https://github.com/oxc-project/oxc/pull/3645>
        // experimentalDecorators: true + emitDecoratorMetadata: true
        // (
        //     "
        //     import Foo from 'foo';
        //     @deco
        //     class A {
        //       constructor(foo: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import Foo from 'foo';
        //     class A {
        //       @deco
        //       foo: Foo;
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import Foo from 'foo';
        //     class A {
        //       @deco
        //       foo(foo: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import Foo from 'foo';
        //     class A {
        //       @deco
        //       foo(): Foo {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import Foo from 'foo';
        //     class A {
        //       foo(@deco foo: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import Foo from 'foo';
        //     class A {
        //       @deco
        //       set foo(value: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import Foo from 'foo';
        //     class A {
        //       @deco
        //       get foo() {}

        //       set foo(value: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import Foo from 'foo';
        //     class A {
        //       @deco
        //       get foo() {}

        //       set ['foo'](value: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import type { Foo } from 'foo';
        //     const key = 'k';
        //     class A {
        //       @deco
        //       get [key]() {}

        //       set [key](value: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import * as foo from 'foo';
        //     @deco
        //     class A {
        //       constructor(foo: foo.Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import type { ClassA } from './classA';

        //     export class ClassB {
        //       public constructor(node: ClassA) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import type Foo from 'foo';
        //     @deco
        //     class A {
        //       constructor(foo: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import type { Foo } from 'foo';
        //     @deco
        //     class A {
        //       constructor(foo: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import type { Type } from 'foo';
        //     import { Foo, Bar } from 'foo';
        //     @deco
        //     class A {
        //       constructor(foo: Foo) {}
        //     }
        //     type T = Bar;
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import { V } from 'foo';
        //     import type { Foo, Bar, T } from 'foo';
        //     @deco
        //     class A {
        //       constructor(foo: Foo) {}
        //       foo(@deco bar: Bar) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import type { Foo, T } from 'foo';
        //     import { V } from 'foo';
        //     @deco
        //     class A {
        //       constructor(foo: Foo) {}
        //     }
        //   ",
        //     None,
        // ),
        // (
        //     "
        //     import type * as Type from 'foo';
        //     @deco
        //     class A {
        //       constructor(foo: Type.Foo) {}
        //     }
        //   ",
        //     None,
        // ),
    ];

    let fail = vec![
        (
            "
              import Foo from 'foo';
              let foo: Foo;
              type Bar = Foo;
              interface Baz {
                foo: Foo;
              }
              function fn(a: Foo): Foo {}
            ",
            None,
        ),
        (
            "
              import Foo from 'foo';
              let foo: Foo;
            ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            "
              import Foo from 'foo';
              let foo: Foo;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { A, B } from 'foo';
              let foo: A;
              let bar: B;
            ",
            None,
        ),
        (
            "
              import { A as a, B as b } from 'foo';
              let foo: a;
              let bar: b;
            ",
            None,
        ),
        (
            "
              import Foo from 'foo';
              type Bar = typeof Foo; // TSTypeQuery
            ",
            None,
        ),
        (
            "
              import foo from 'foo';
              type Bar = foo.Bar; // TSQualifiedName
            ",
            None,
        ),
        (
            "
              import foo from 'foo';
              type Baz = (typeof foo.bar)['Baz']; // TSQualifiedName & TSTypeQuery
            ",
            None,
        ),
        (
            "
              import * as A from 'foo';
              let foo: A.Foo;
            ",
            None,
        ),
        (
            "
              import A, { B } from 'foo';
              let foo: A;
              let bar: B;
            ",
            None,
        ),
        (
            "
              import A, {} from 'foo';
              let foo: A;
            ",
            None,
        ),
        (
            "
              import { A, B } from 'foo';
              const foo: A = B();
            ",
            None,
        ),
        (
            "
              import { A, B, C } from 'foo';
              const foo: A = B();
              let bar: C;
            ",
            None,
        ),
        (
            "
              import { A, B, C, D } from 'foo';
              const foo: A = B();
              type T = { bar: C; baz: D };
            ",
            None,
        ),
        (
            "
              import A, { B, C, D } from 'foo';
              B();
              type T = { foo: A; bar: C; baz: D };
            ",
            None,
        ),
        (
            "
              import A, { B } from 'foo';
              B();
              type T = A;
            ",
            None,
        ),
        (
            "
              import type Already1Def from 'foo';
              import type { Already1 } from 'foo';
              import A, { B } from 'foo';
              import { C, D, E } from 'bar';
              import type { Already2 } from 'bar';
              type T = { b: B; c: C; d: D };
            ",
            None,
        ),
        (
            "
              import A, { /* comment */ B } from 'foo';
              type T = B;
            ",
            None,
        ),
        (
            "
              import { A, B, C } from 'foo';
              import { D, E, F, } from 'bar';
              type T = A | D;
            ",
            None,
        ),
        (
            "
              import { A, B, C } from 'foo';
              import { D, E, F, } from 'bar';
              type T = B | E;
            ",
            None,
        ),
        (
            "
              import { A, B, C } from 'foo';
              import { D, E, F, } from 'bar';
              type T = C | F;
            ",
            None,
        ),
        (
            "
              import { Type1, Type2 } from 'named_types';
              import Type from 'default_type';
              import * as Types from 'namespace_type';
              import Default, { Named } from 'default_and_named_type';
              type T = Type1 | Type2 | Type | Types.A | Default | Named;
            ",
            None,
        ),
        (
            "
              import { Value1, Type1 } from 'named_import';
              import Type2, { Value2 } from 'default_import';
              import Value3, { Type3 } from 'default_import2';
              import Type4, { Type5, Value4 } from 'default_and_named_import';
              type T = Type1 | Type2 | Type3 | Type4 | Type5;
            ",
            None,
        ),
        (
            "
              let foo: import('foo');
              let bar: import('foo').Bar;
            ",
            None,
        ),
        (
            "
              let foo: import('foo');
            ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            "
              import type Foo from 'foo';
              let foo: Foo;
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            "
              import type { Foo } from 'foo';
              let foo: Foo;
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        // (
        //     "
        //       import Type from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       import { Type } from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       import * as Type from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       import type Type from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        // ),
        // (
        //     "
        //       import type { Type } from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        // ),
        // (
        //     "
        //       import type * as Type from 'foo';

        //       type T = typeof Type;
        //       type T = typeof Type.foo;
        //     ",
        //     Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        // ),
        (
            "
              import Type from 'foo';
        
              export type { Type }; // is a type-only export
            ",
            None,
        ),
        (
            "
              import { Type } from 'foo';
        
              export type { Type }; // is a type-only export
            ",
            None,
        ),
        (
            "
              import * as Type from 'foo';
        
              export type { Type }; // is a type-only export
            ",
            None,
        ),
        (
            "
              import type Type from 'foo';
        
              export { Type }; // is a type-only export
              export default Type; // is a type-only export
              export type { Type }; // is a type-only export
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            "
              import type { Type } from 'foo';
        
              export { Type }; // is a type-only export
              export default Type; // is a type-only export
              export type { Type }; // is a type-only export
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            "
              import type * as Type from 'foo';
              
              export { Type }; // is a type-only export
              export default Type; // is a type-only export
              export type { Type }; // is a type-only export
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            "
              import type /*comment*/ * as AllType from 'foo';
              import type // comment
              DefType from 'foo';
              import type /*comment*/ { Type } from 'foo';
              
              type T = { a: AllType; b: DefType; c: Type };
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            "
              import Default, * as Rest from 'module';
              const a: Rest.A = '';
            ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            "
              import Default, * as Rest from 'module';
              const a: Default = '';
            ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            "
              import Default, * as Rest from 'module';
              const a: Default = '';
              const b: Rest.A = '';
            ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            "
              import Default, /*comment*/ * as Rest from 'module';
              const a: Default = '';
            ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            "
              import Default /*comment1*/, /*comment2*/ { Data } from 'module';
              const a: Default = '';
            ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        // (
        //     "
        //       import Foo from 'foo';
        //       @deco
        //       class A {
        //         constructor(foo: Foo) {}
        //       }
        //     ",
        //     None,
        // ),
        (
            "
              import { type A, B } from 'foo';
              type T = A;
              const b = B;
            ",
            Some(serde_json::json!([{ "prefer": "no-type-imports" }])),
        ),
        (
            "
              import { A, B, type C } from 'foo';
              type T = A | C;
              const b = B;
            ",
            Some(serde_json::json!([{ "prefer": "type-imports" }])),
        ),
        (
            "
              import { A, B } from 'foo';
              let foo: A;
              let bar: B;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { A, B } from 'foo';
              
              let foo: A;
              B();
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { A, B } from 'foo';
              type T = A;
              B();
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { A } from 'foo';
              import { B } from 'foo';
              type T = A;
              type U = B;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { A } from 'foo';
              import B from 'foo';
              type T = A;
              type U = B;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import A, { B, C } from 'foo';
              type T = B;
              type U = C;
              A();
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import A, { B, C } from 'foo';
              type T = B;
              type U = C;
              type V = A;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import A, { B, C as D } from 'foo';
              type T = B;
              type U = D;
              type V = A;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { /* comment */ A, B } from 'foo';
              type T = A;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { B, /* comment */ A } from 'foo';
              type T = A;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { A, B, C } from 'foo';
              import type { D } from 'deez';
              
              const foo: A = B();
              let bar: C;
              let baz: D;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { A, B, type C } from 'foo';
              import type { D } from 'deez';
              const foo: A = B();
              let bar: C;
              let baz: D;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import A from 'foo';
              export = {} as A;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        (
            "
              import { A } from 'foo';
              export = {} as A;
            ",
            Some(
                serde_json::json!([            { "prefer": "type-imports", "fixStyle": "inline-type-imports" },          ]),
            ),
        ),
        // (
        //     "
        //         import Foo from 'foo';
        //         @deco
        //         class A {
        //           constructor(foo: Foo) {}
        //         }
        //     ",
        //     None,
        // ),
        // (
        //     "
        //         import Foo from 'foo';
        //         class A {
        //           @deco
        //           foo: Foo;
        //         }
        //     ",
        //     None,
        // ),
        // (
        //     "
        //         import Foo from 'foo';
        //         class A {
        //           @deco
        //           foo(foo: Foo) {}
        //         }
        //     ",
        //     None,
        // ),
        (
            "
                import Foo from 'foo';
                class A {
                  @deco
                  foo(): Foo {}
                }
            ",
            None,
        ),
        (
            "
                import Foo from 'foo';
                class A {
                  foo(@deco foo: Foo) {}
                }
            ",
            None,
        ),
        (
            "
                import Foo from 'foo';
                class A {
                  @deco
                  set foo(value: Foo) {}
                }
            ",
            None,
        ),
        (
            "
                import Foo from 'foo';
                class A {
                  @deco
                  get foo() {}
              
                  set foo(value: Foo) {}
                }
            ",
            None,
        ),
        (
            "
                import Foo from 'foo';
                class A {
                  @deco
                  get foo() {}
              
                  set ['foo'](value: Foo) {}
                }
            ",
            None,
        ),
        // (
        //     "
        //         import * as foo from 'foo';
        //         @deco
        //         class A {
        //           constructor(foo: foo.Foo) {}
        //         }
        //     ",
        //     None,
        // ),
        (
            "
              import 'foo';
              import { Foo, Bar } from 'foo';
              function test(foo: Foo) {}
            ",
            None,
        ),
        (
            "
              import {} from 'foo';
              import { Foo, Bar } from 'foo';
              function test(foo: Foo) {}
            ",
            None,
        ),
        // experimentalDecorators: true + emitDecoratorMetadata: true
        // (
        //     "
        //       import Foo from 'foo';
        //       export type T = Foo;
        //     ",
        //     None,
        // ),
    ];

    // let fix = vec![
    // (
    //     "
    //       import Foo from 'foo';
    //       let foo: Foo;
    //       type Bar = Foo;
    //       interface Baz {
    //         foo: Foo;
    //       }
    //       function fn(a: Foo): Foo {}
    //     ",
    //     "
    //       import type Foo from 'foo';
    //       let foo: Foo;
    //       type Bar = Foo;
    //       interface Baz {
    //         foo: Foo;
    //       }
    //       function fn(a: Foo): Foo {}
    //     ",
    //     None,
    // ),
    //     (
    //         "
    //           import Foo from 'foo';
    //           let foo: Foo;
    //         ",
    //         "
    //           import type Foo from 'foo';
    //           let foo: Foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Foo from 'foo';
    //           let foo: Foo;
    //         ",
    //         "
    //           import type Foo from 'foo';
    //           let foo: Foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B } from 'foo';
    //           let foo: A;
    //           let bar: B;
    //         ",
    //         "
    //           import type { A, B } from 'foo';
    //           let foo: A;
    //           let bar: B;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A as a, B as b } from 'foo';
    //           let foo: a;
    //           let bar: b;
    //         ",
    //         "
    //           import type { A as a, B as b } from 'foo';
    //           let foo: a;
    //           let bar: b;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Foo from 'foo';
    //           type Bar = typeof Foo; // TSTypeQuery
    //         ",
    //         "
    //           import type Foo from 'foo';
    //           type Bar = typeof Foo; // TSTypeQuery
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import foo from 'foo';
    //           type Bar = foo.Bar; // TSQualifiedName
    //         ",
    //         "
    //           import type foo from 'foo';
    //           type Bar = foo.Bar; // TSQualifiedName
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import foo from 'foo';
    //           type Baz = (typeof foo.bar)['Baz']; // TSQualifiedName & TSTypeQuery
    //         ",
    //         "
    //           import type foo from 'foo';
    //           type Baz = (typeof foo.bar)['Baz']; // TSQualifiedName & TSTypeQuery
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import * as A from 'foo';
    //           let foo: A.Foo;
    //         ",
    //         "
    //           import type * as A from 'foo';
    //           let foo: A.Foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import A, { B } from 'foo';
    //           let foo: A;
    //           let bar: B;
    //         ",
    //         "
    //           import type { B } from 'foo';
    //           import type A from 'foo';
    //           let foo: A;
    //           let bar: B;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import A, {} from 'foo';
    //           let foo: A;
    //         ",
    //         "
    //           import type A from 'foo';
    //           let foo: A;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B } from 'foo';
    //           const foo: A = B();
    //         ",
    //         "
    //           import type { A} from 'foo';
    //           import { B } from 'foo';
    //           const foo: A = B();
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B, C } from 'foo';
    //           const foo: A = B();
    //           let bar: C;
    //         ",
    //         "
    //           import type { A, C } from 'foo';
    //           import { B } from 'foo';
    //           const foo: A = B();
    //           let bar: C;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B, C, D } from 'foo';
    //           const foo: A = B();
    //           type T = { bar: C; baz: D };
    //         ",
    //         "
    //           import type { A, C, D } from 'foo';
    //           import { B } from 'foo';
    //           const foo: A = B();
    //           type T = { bar: C; baz: D };
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import A, { B, C, D } from 'foo';
    //           B();
    //           type T = { foo: A; bar: C; baz: D };
    //         ",
    //         "
    //           import type { C, D } from 'foo';
    //           import type A from 'foo';
    //           import { B } from 'foo';
    //           B();
    //           type T = { foo: A; bar: C; baz: D };
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import A, { B } from 'foo';
    //           B();
    //           type T = A;
    //         ",
    //         "
    //           import type A from 'foo';
    //           import { B } from 'foo';
    //           B();
    //           type T = A;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type Already1Def from 'foo';
    //           import type { Already1 } from 'foo';
    //           import A, { B } from 'foo';
    //           import { C, D, E } from 'bar';
    //           import type { Already2 } from 'bar';
    //           type T = { b: B; c: C; d: D };
    //         ",
    //         "
    //           import type Already1Def from 'foo';
    //           import type { Already1 , B } from 'foo';
    //           import A from 'foo';
    //           import { E } from 'bar';
    //           import type { Already2 , C, D} from 'bar';
    //           type T = { b: B; c: C; d: D };
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import A, { /* comment */ B } from 'foo';
    //           type T = B;
    //         ",
    //         "
    //           import type { /* comment */ B } from 'foo';
    //           import A from 'foo';
    //           type T = B;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B, C } from 'foo';
    //           import { D, E, F, } from 'bar';
    //           type T = A | D;
    //         ",
    //         "
    //           import type { A} from 'foo';
    //           import { B, C } from 'foo';
    //           import type { D} from 'bar';
    //           import { E, F, } from 'bar';
    //           type T = A | D;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B, C } from 'foo';
    //           import { D, E, F, } from 'bar';
    //           type T = B | E;
    //         ",
    //         "
    //           import type { B} from 'foo';
    //           import { A, C } from 'foo';
    //           import type { E} from 'bar';
    //           import { D, F, } from 'bar';
    //           type T = B | E;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B, C } from 'foo';
    //           import { D, E, F, } from 'bar';
    //           type T = C | F;
    //         ",
    //         "
    //           import type { C } from 'foo';
    //           import { A, B } from 'foo';
    //           import type { F} from 'bar';
    //           import { D, E } from 'bar';
    //           type T = C | F;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { Type1, Type2 } from 'named_types';
    //           import Type from 'default_type';
    //           import * as Types from 'namespace_type';
    //           import Default, { Named } from 'default_and_named_type';
    //           type T = Type1 | Type2 | Type | Types.A | Default | Named;
    //         ",
    //         "
    //           import type { Type1, Type2 } from 'named_types';
    //           import type Type from 'default_type';
    //           import type * as Types from 'namespace_type';
    //           import type { Named } from 'default_and_named_type';
    //           import type Default from 'default_and_named_type';
    //           type T = Type1 | Type2 | Type | Types.A | Default | Named;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { Value1, Type1 } from 'named_import';
    //           import Type2, { Value2 } from 'default_import';
    //           import Value3, { Type3 } from 'default_import2';
    //           import Type4, { Type5, Value4 } from 'default_and_named_import';
    //           type T = Type1 | Type2 | Type3 | Type4 | Type5;
    //         ",
    //         "
    //           import type { Type1 } from 'named_import';
    //           import { Value1 } from 'named_import';
    //           import type Type2 from 'default_import';
    //           import { Value2 } from 'default_import';
    //           import type { Type3 } from 'default_import2';
    //           import Value3 from 'default_import2';
    //           import type { Type5} from 'default_and_named_import';
    //           import type Type4 from 'default_and_named_import';
    //           import { Value4 } from 'default_and_named_import';
    //           type T = Type1 | Type2 | Type3 | Type4 | Type5;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type Foo from 'foo';
    //           let foo: Foo;
    //         ",
    //         "
    //           import Foo from 'foo';
    //           let foo: Foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type { Foo } from 'foo';
    //           let foo: Foo;
    //         ",
    //         "
    //           import { Foo } from 'foo';
    //           let foo: Foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Type from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         "
    //           import type Type from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { Type } from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         "
    //           import type { Type } from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import * as Type from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         "
    //           import type * as Type from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type Type from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         "
    //           import Type from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type { Type } from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         "
    //           import { Type } from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type * as Type from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         "
    //           import * as Type from 'foo';
    //
    //           type T = typeof Type;
    //           type T = typeof Type.foo;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Type from 'foo';
    //
    //           export type { Type }; // is a type-only export
    //         ",
    //         "
    //           import type Type from 'foo';
    //
    //           export type { Type }; // is a type-only export
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { Type } from 'foo';
    //
    //           export type { Type }; // is a type-only export
    //         ",
    //         "
    //           import type { Type } from 'foo';
    //
    //           export type { Type }; // is a type-only export
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import * as Type from 'foo';
    //
    //           export type { Type }; // is a type-only export
    //         ",
    //         "
    //           import type * as Type from 'foo';
    //
    //           export type { Type }; // is a type-only export
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type Type from 'foo';
    //
    //           export { Type }; // is a type-only export
    //           export default Type; // is a type-only export
    //           export type { Type }; // is a type-only export
    //         ",
    //         "
    //           import Type from 'foo';
    //
    //           export { Type }; // is a type-only export
    //           export default Type; // is a type-only export
    //           export type { Type }; // is a type-only export
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type { Type } from 'foo';
    //
    //           export { Type }; // is a type-only export
    //           export default Type; // is a type-only export
    //           export type { Type }; // is a type-only export
    //         ",
    //         "
    //           import { Type } from 'foo';
    //
    //           export { Type }; // is a type-only export
    //           export default Type; // is a type-only export
    //           export type { Type }; // is a type-only export
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type * as Type from 'foo';
    //
    //           export { Type }; // is a type-only export
    //           export default Type; // is a type-only export
    //           export type { Type }; // is a type-only export
    //         ",
    //         "
    //           import * as Type from 'foo';
    //
    //           export { Type }; // is a type-only export
    //           export default Type; // is a type-only export
    //           export type { Type }; // is a type-only export
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import type /*comment*/ * as AllType from 'foo';
    //           import type // comment
    //           DefType from 'foo';
    //           import type /*comment*/ { Type } from 'foo';
    //
    //           type T = { a: AllType; b: DefType; c: Type };
    //         ",
    //         "
    //           import /*comment*/ * as AllType from 'foo';
    //           import // comment
    //           DefType from 'foo';
    //           import /*comment*/ { Type } from 'foo';
    //
    //           type T = { a: AllType; b: DefType; c: Type };
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Default, * as Rest from 'module';
    //           const a: Rest.A = '';
    //         ",
    //         "
    //           import type * as Rest from 'module';
    //           import Default from 'module';
    //           const a: Rest.A = '';
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Default, * as Rest from 'module';
    //           const a: Default = '';
    //         ",
    //         "
    //           import type Default from 'module';
    //           import * as Rest from 'module';
    //           const a: Default = '';
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Default, * as Rest from 'module';
    //           const a: Default = '';
    //           const b: Rest.A = '';
    //         ",
    //         "
    //           import type * as Rest from 'module';
    //           import type Default from 'module';
    //           const a: Default = '';
    //           const b: Rest.A = '';
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Default, /*comment*/ * as Rest from 'module';
    //           const a: Default = '';
    //         ",
    //         "
    //           import type Default from 'module';
    //           import /*comment*/ * as Rest from 'module';
    //           const a: Default = '';
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Default /*comment1*/, /*comment2*/ { Data } from 'module';
    //           const a: Default = '';
    //         ",
    //         "
    //           import type Default /*comment1*/ from 'module';
    //           import /*comment2*/ { Data } from 'module';
    //           const a: Default = '';
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Foo from 'foo';
    //           @deco
    //           class A {
    //             constructor(foo: Foo) {}
    //           }
    //         ",
    //         "
    //           import type Foo from 'foo';
    //           @deco
    //           class A {
    //             constructor(foo: Foo) {}
    //           }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { type A, B } from 'foo';
    //           type T = A;
    //           const b = B;
    //         ",
    //         "
    //           import { A, B } from 'foo';
    //           type T = A;
    //           const b = B;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B, type C } from 'foo';
    //           type T = A | C;
    //           const b = B;
    //         ",
    //         "
    //           import type { A} from 'foo';
    //           import { B, type C } from 'foo';
    //           type T = A | C;
    //           const b = B;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B } from 'foo';
    //           let foo: A;
    //           let bar: B;
    //         ",
    //         "
    //           import { type A, type B } from 'foo';
    //           let foo: A;
    //           let bar: B;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B } from 'foo';
    //
    //           let foo: A;
    //           B();
    //         ",
    //         "
    //           import { type A, B } from 'foo';
    //
    //           let foo: A;
    //           B();
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B } from 'foo';
    //           type T = A;
    //           B();
    //         ",
    //         "
    //           import { type A, B } from 'foo';
    //           type T = A;
    //           B();
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A } from 'foo';
    //           import { B } from 'foo';
    //           type T = A;
    //           type U = B;
    //         ",
    //         "
    //           import { type A } from 'foo';
    //           import { type B } from 'foo';
    //           type T = A;
    //           type U = B;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A } from 'foo';
    //           import B from 'foo';
    //           type T = A;
    //           type U = B;
    //         ",
    //         "
    //           import { type A } from 'foo';
    //           import type B from 'foo';
    //           type T = A;
    //           type U = B;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import A, { B, C } from 'foo';
    //           type T = B;
    //           type U = C;
    //           A();
    //         ",
    //         "
    //           import A, { type B, type C } from 'foo';
    //           type T = B;
    //           type U = C;
    //           A();
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import A, { B, C } from 'foo';
    //           type T = B;
    //           type U = C;
    //           type V = A;
    //         ",
    //         "
    //           import {type B, type C} from 'foo';
    //           import type A from 'foo';
    //           type T = B;
    //           type U = C;
    //           type V = A;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import A, { B, C as D } from 'foo';
    //           type T = B;
    //           type U = D;
    //           type V = A;
    //         ",
    //         "
    //           import {type B, type C as D} from 'foo';
    //           import type A from 'foo';
    //           type T = B;
    //           type U = D;
    //           type V = A;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { /* comment */ A, B } from 'foo';
    //           type T = A;
    //         ",
    //         "
    //           import { /* comment */ type A, B } from 'foo';
    //           type T = A;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { B, /* comment */ A } from 'foo';
    //           type T = A;
    //         ",
    //         "
    //           import { B, /* comment */ type A } from 'foo';
    //           type T = A;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B, C } from 'foo';
    //           import type { D } from 'deez';
    //
    //           const foo: A = B();
    //           let bar: C;
    //           let baz: D;
    //         ",
    //         "
    //           import { type A, B, type C } from 'foo';
    //           import type { D } from 'deez';
    //
    //           const foo: A = B();
    //           let bar: C;
    //           let baz: D;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A, B, type C } from 'foo';
    //           import type { D } from 'deez';
    //           const foo: A = B();
    //           let bar: C;
    //           let baz: D;
    //         ",
    //         "
    //           import { type A, B, type C } from 'foo';
    //           import type { D } from 'deez';
    //           const foo: A = B();
    //           let bar: C;
    //           let baz: D;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import A from 'foo';
    //           export = {} as A;
    //         ",
    //         "
    //           import type A from 'foo';
    //           export = {} as A;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import { A } from 'foo';
    //           export = {} as A;
    //         ",
    //         "
    //           import { type A } from 'foo';
    //           export = {} as A;
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //             import Foo from 'foo';
    //             /@deco
    //             class A {
    //               constructor(foo: Foo) {}
    //             }
    //         ",
    //         "
    //             import type Foo from 'foo';
    //             @deco
    //             class A {
    //               constructor(foo: Foo) {}
    //             }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //             import Foo from 'foo';
    //             class A {
    //               @deco
    //               foo: Foo;
    //             }
    //         ",
    //         "
    //             import type Foo from 'foo';
    //             class A {
    //               @deco
    //               foo: Foo;
    //             }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //             import Foo from 'foo';
    //             class A {
    //               @deco
    //               foo(foo: Foo) {}
    //             }
    //         ",
    //         "
    //             import type Foo from 'foo';
    //             class A {
    //               @deco
    //               foo(foo: Foo) {}
    //             }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //             import Foo from 'foo';
    //             class A {
    //               @deco
    //               foo(): Foo {}
    //             }
    //         ",
    //         "
    //             import type Foo from 'foo';
    //             class A {
    //               @deco
    //               foo(): Foo {}
    //             }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //             import Foo from 'foo';
    //             class A {
    //               foo(@deco foo: Foo) {}
    //             }
    //         ",
    //         "
    //             import type Foo from 'foo';
    //             class A {
    //               foo(@deco foo: Foo) {}
    //             }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //             import Foo from 'foo';
    //             class A {
    //               @deco
    //               set foo(value: Foo) {}
    //             }
    //         ",
    //         "
    //             import type Foo from 'foo';
    //             class A {
    //               @deco
    //               set foo(value: Foo) {}
    //             }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //             import Foo from 'foo';
    //             class A {
    //               @deco
    //               get foo() {}
    //
    //               set foo(value: Foo) {}
    //             }
    //         ",
    //         "
    //             import type Foo from 'foo';
    //             class A {
    //               @deco
    //               get foo() {}
    //
    //               set foo(value: Foo) {}
    //             }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //             import Foo from 'foo';
    //             class A {
    //               @deco
    //               get foo() {}
    //
    //               set ['foo'](value: Foo) {}
    //             }
    //         ",
    //         "
    //             import type Foo from 'foo';
    //             class A {
    //               @deco
    //               get foo() {}
    //
    //               set ['foo'](value: Foo) {}
    //             }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //             import * as foo from 'foo';
    //             @deco
    //             class A {
    //               constructor(foo: foo.Foo) {}
    //             }
    //         ",
    //         "
    //             import type * as foo from 'foo';
    //             @deco
    //             class A {
    //               constructor(foo: foo.Foo) {}
    //             }
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import 'foo';
    //           import { Foo, Bar } from 'foo';
    //           function test(foo: Foo) {}
    //         ",
    //         "
    //           import 'foo';
    //           import type { Foo} from 'foo';
    //           import { Bar } from 'foo';
    //           function test(foo: Foo) {}
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import {} from 'foo';
    //           import { Foo, Bar } from 'foo';
    //           function test(foo: Foo) {}
    //         ",
    //         "
    //           import {} from 'foo';
    //           import type { Foo} from 'foo';
    //           import { Bar } from 'foo';
    //           function test(foo: Foo) {}
    //         ",
    //         None,
    //     ),
    //     (
    //         "
    //           import Foo from 'foo';
    //           export type T = Foo;
    //         ",
    //         "
    //           import type Foo from 'foo';
    //           export type T = Foo;
    //         ",
    //         None,
    //     ),
    // ];
    Tester::new(ConsistentTypeImports::NAME, pass, fail).test_and_snapshot();
}
