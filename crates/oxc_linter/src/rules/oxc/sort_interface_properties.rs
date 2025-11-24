use itertools::all;
use oxc_ast::{
    AstKind,
    ast::{PropertyKey, TSSignature},
};

use crate::{AstNode, context::LintContext, rule::Rule};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require that interface properties are sorted.
    ///
    /// ### Why is this bad?
    ///
    /// A consistent ordering of properties can make code more readable and maintainable.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```typescript
    /// zProperty: string;
    /// aProperty: number;
    /// new (): any;
    /// (): void;
    /// yMethod(): boolean;
    /// [index: number]: string;
    /// bMethod(): string;
    /// [key: string]: any;
    /// cField: object;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```typescript
    /// aProperty: number;
    /// bMethod(): string;
    /// cField: object;
    /// yMethod(): boolean;
    /// zProperty: string;
    /// new (): any;
    /// (): void;
    /// [index: number]: string;
    /// [key: string]: any;
    /// ```
    SortInterfaceProperties,
    oxc,
    style,
    conditional_fix,
);

#[derive(Debug, Default, Clone)]
pub struct SortInterfaceProperties;

#[derive(Clone)]
struct MemberInfo {
    name: Option<String>, // None for non-sortable members
    span: Span,
}
fn sort_interface_properties_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Interface properties should be sorted").with_label(span)
}
impl SortInterfaceProperties {
    fn check_members<'a>(
        members: &oxc_allocator::Vec<'a, TSSignature<'a>>,
        span: Span,
        ctx: &LintContext<'a>,
    ) {
        if members.len() < 2 {
            return;
        }

        let mut all_members: Vec<MemberInfo> = Vec::with_capacity(members.len());

        for member in members {
            all_members
                .push(MemberInfo { name: Self::get_member_name(member), span: member.span() });
        }

        // Extract just the sortable member names
        let sortable_names: Vec<&String> =
            all_members.iter().filter_map(|m| m.name.as_ref()).collect();

        if sortable_names.len() < 2 {
            return; // Nothing to sort
        }

        // Check if sortable members are in order
        let mut sorted_names = sortable_names.clone();
        sorted_names.sort_unstable();

        let is_sorted = all(sortable_names.iter().zip(&sorted_names), |(a, b)| a == b);

        if !is_sorted {
            // Check for comments within the entire interface body
            // We cannot safely reorder if there are any comments
            let can_fix = !ctx.has_comments_between(span);

            if can_fix {
                // Strategy: sort sortable members, then append other members to the end
                Self::fix_members(&all_members, span, ctx);
                return;
            }

            // Fallback: emit diagnostic without fix
            ctx.diagnostic(sort_interface_properties_diagnostic(span));
        }
    }

    fn fix_members(all_members: &[MemberInfo], span: Span, ctx: &LintContext) {
        // Separate sortable and non-sortable members
        let mut sortable: Vec<&MemberInfo> =
            all_members.iter().filter(|m| m.name.is_some()).collect();

        let non_sortable: Vec<&MemberInfo> =
            all_members.iter().filter(|m| m.name.is_none()).collect();

        // Sort sortable members by name
        sortable.sort_by(|a, b| a.name.as_ref().unwrap().cmp(b.name.as_ref().unwrap()));

        // Build text for each member
        let mut member_texts: Vec<String> = Vec::with_capacity(all_members.len());

        for i in 0..all_members.len() {
            let start = all_members[i].span.start;
            let end = if i + 1 < all_members.len() {
                all_members[i + 1].span.start
            } else {
                all_members[i].span.end
            };
            member_texts.push(ctx.source_range(Span::new(start, end)).to_string());
        }

        // Build final text: sorted members + non-sortables
        let mut sorted_text = String::new();
        let trim_semicolon = |s: &str| -> String {
            let s = s.trim_end();
            let mut trimmed = s.to_string();
            if trimmed.ends_with(';') {
                trimmed.pop();
                trimmed = trimmed.trim_end().to_string();
            }
            trimmed
        };

        let total_members = sortable.len() + non_sortable.len();
        let mut current = 0;

        // Add sorted members
        for member in &sortable {
            let idx = all_members.iter().position(|m| m.span == member.span).unwrap();
            let is_last = current + 1 == total_members;
            let text = &member_texts[idx];

            current += 1;
            sorted_text.push_str(&trim_semicolon(text));

            if is_last {
                sorted_text.push(';');
            } else {
                sorted_text.push_str(";\n");
            }
        }

        // Add non-sortable members at the end
        for member in &non_sortable {
            let idx = all_members.iter().position(|m| m.span == member.span).unwrap();
            let is_last = current + 1 == total_members;
            let text = &member_texts[idx];

            current += 1;
            sorted_text.push_str(&trim_semicolon(text));

            if is_last {
                sorted_text.push(';');
            } else {
                sorted_text.push_str(";\n");
            }
        }

        // Replace all members
        let replace_span =
            Span::new(all_members[0].span.start, all_members.last().unwrap().span.end);

        ctx.diagnostic_with_fix(sort_interface_properties_diagnostic(span), |fixer| {
            fixer.replace(replace_span, sorted_text)
        });
    }
    fn get_member_name(member: &TSSignature) -> Option<String> {
        match member {
            TSSignature::TSPropertySignature(prop) => Self::get_property_key_name(&prop.key),
            TSSignature::TSMethodSignature(method) => Self::get_property_key_name(&method.key),
            TSSignature::TSCallSignatureDeclaration(_)
            | TSSignature::TSConstructSignatureDeclaration(_)
            | TSSignature::TSIndexSignature(_) => None,
        }
    }
    fn get_property_key_name(key: &PropertyKey) -> Option<String> {
        match key {
            PropertyKey::StaticIdentifier(ident) => Some(ident.name.to_string()),
            PropertyKey::PrivateIdentifier(ident) => Some(ident.name.to_string()),
            PropertyKey::StringLiteral(lit) => Some(lit.value.to_string()),
            PropertyKey::NumericLiteral(lit) => {
                lit.raw.as_ref().map(std::string::ToString::to_string)
            }
            _ => None,
        }
    }
}

impl Rule for SortInterfaceProperties {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::TSInterfaceBody(interface_body) => {
                Self::check_members(&interface_body.body, node.span(), ctx);
            }
            AstKind::TSTypeLiteral(type_literal) => {
                Self::check_members(&type_literal.members, node.span(), ctx);
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        // Already sorted interface
        "interface Foo {
            a: number;
            b: string;
        }",
        // Single member
        "interface Foo { a: number; }",
        // With methods
        "interface Foo {
            a: number;
            b(): void;
        }",
        // Already sorted with various key types
        "interface AlreadySorted {
            aProperty: string;
            bMethod(): number;
            cField: boolean;
            dGetter: object;
        }",
        // Numeric literals sorted lexicographically (11 < 5 < 99 as strings)
        "interface NumericKeys {
            11: string;
            5: number;
            99: string;
        }",
        // String literals sorted
        "interface StringKeys {
            \"aa\": string;
            \"zz\": string;
        }",
        // Non-sortable members (index signatures, call/construct signatures) are skipped
        "interface WithNonSortable {
            [index: number]: any;
            (): void;
            new (): any;
        }",
        "interface _AlreadySorted {
            aProperty: number;
            bMethod(): string;
            cField: object;
            yMethod(): boolean;
            zProperty: string;
            new (): any;
            (): void;
            [index: number]: string;
            [key: string]: any;
        }",
        // Complex interface with JSDoc and various comment styles (already sorted)
        "interface _WithComments {
            // Simple comment for apple
            apple: number;

            // Simple comment for banana
            banana: boolean; // inline comment

            /* Block comment for charlie */
            charlie: object;

            /**
             * Zebra property with JSDoc comment
             * @example zebra = 'test'
             */
            zebra: string;
        }",
    ];

    let fail = vec![
        // Unsorted interface
        "interface Foo {
            b: string;
            a: number;
        }",
        // Multiple members unsorted
        "interface Foo {
            c: number;
            a: string;
            b: boolean;
        }",
        // Unsorted with comment (no fix due to comments)
        "interface _WithSimpleComment {
            // Comment for b
            b: string;
            a: number;
        }",
        // Numeric literals unsorted
        "interface NumericKeys {
            99: string;
            11: string;
            5: number;
        }",
        // String literals unsorted
        "interface Foo {
            \"zz\": string;
            \"aa\": string;
        }",
        // Mixed key types unsorted
        "interface MixedKeys {
            \"zebra\": string;
            apple: number;
            99: boolean;
            \"banana\": string;
            5: number;
            charlie: object;
        }",
        "interface MixedSortable {
            zProperty: string;
            aProperty: number;
            new (): any;
            (): void;
            yMethod(): boolean;
            [index: number]: string;
            bMethod(): string;
            [key: string]: any;
            cField: object;
        }",
        // Complex unsorted interface with JSDoc and various comment styles
        "interface _WithComments {
            // This is a zebra property
            zebra: string;

            /**
             * Apple property with JSDoc comment
             * @example apple = 22
             */
            apple: number;

            // Simple comment for banana
            banana: boolean; // inline comment

            /* Block comment for charlie */
            charlie: object;
        }",
    ];

    let fix = vec![
        // Basic sorting
        ("interface Foo { b: string; a: number; }", "interface Foo { a: number;\nb: string; }"),
        // Three properties
        (
            "interface Foo { c: number; a: string; b: boolean; }",
            "interface Foo { a: string;\nb: boolean;\nc: number; }",
        ),
        // Numeric literals (lexicographic sort: "11" < "5" < "99")
        (
            "interface NumericKeys { 99: string; 11: string; 5: number; }",
            "interface NumericKeys { 11: string;\n5: number;\n99: string; }",
        ),
        // String literals
        (
            "interface Foo { \"zz\": string; \"aa\": string; }",
            "interface Foo { \"aa\": string;\n\"zz\": string; }",
        ),
    ];

    Tester::new(SortInterfaceProperties::NAME, SortInterfaceProperties::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
