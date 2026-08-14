use oxc_ast::{
    AstKind,
    ast::{TSTupleElement, TSType},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn consistent_tuple_labels_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("This tuple element should have a label, just like the other elements.")
        .with_help("Add a label to this element, or remove the labels from the other elements.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentTupleLabels;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce consistent labels on tuple type elements: either every element in a
    /// tuple type is labeled, or none of them are.
    ///
    /// ### Why is this bad?
    ///
    /// Labels document what each position of a tuple means. Labeling only some of the
    /// elements leaves the rest unexplained, and the reader cannot tell whether the
    /// missing label was deliberate or forgotten.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// type Foo = [a: string, number];
    /// type Bar = [string, ...rest: number[]];
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// type Foo = [a: string, b: number];
    /// type Bar = [string, number];
    /// type Baz = [a: string, ...b: number[]];
    /// ```
    ConsistentTupleLabels,
    unicorn,
    style,
    version = "next",
    short_description = "Enforce consistent labels on tuple type elements.",
);

/// A rest element keeps its label inside the `TSRestType` wrapper, e.g. `[...rest: number[]]`.
fn is_labeled_element(element: &TSTupleElement) -> bool {
    match element {
        TSTupleElement::TSNamedTupleMember(_) => true,
        TSTupleElement::TSRestType(rest) => {
            matches!(rest.type_annotation, TSType::TSNamedTupleMember(_))
        }
        _ => false,
    }
}

impl Rule for ConsistentTupleLabels {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSTupleType(tuple_type) = node.kind() else {
            return;
        };

        // A tuple with fewer than two elements cannot be inconsistent.
        if tuple_type.element_types.len() < 2 {
            return;
        }

        let unlabeled_count =
            tuple_type.element_types.iter().filter(|element| !is_labeled_element(element)).count();

        // All labeled or all unlabeled is consistent.
        if unlabeled_count == 0 || unlabeled_count == tuple_type.element_types.len() {
            return;
        }

        for element in
            tuple_type.element_types.iter().filter(|element| !is_labeled_element(element))
        {
            ctx.diagnostic(consistent_tuple_labels_diagnostic(element.span()));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "type Foo = [a: string, b: number];",
        "type Foo = [a: string, b: number, c: boolean];",
        "type Foo = [string, number];",
        "type Foo = [string, number, boolean];",
        "type Foo = [];",
        "type Foo = [string];",
        "type Foo = [a: string];",
        "type Foo = [a?: string, b?: number];",
        "type Foo = [a: string, b?: number];",
        "type Foo = [string?, number?];",
        "type Foo = [a: string, ...b: number[]];",
        "type Foo = [string, ...number[]];",
        "type Foo = [[a: number, b: number]];",
        "type Foo = [a: [x: number], b: [y: number]];",
        "type Foo = string[];",
        "type Foo = readonly string[];",
        "type Foo = {a: string; b: number};",
    ];

    let fail = vec![
        "type Foo = [a: string, number];",
        "type Foo = [string, b: number];",
        "type Foo = [a: string, b: number, c: boolean, d];",
        "type Foo = [a: string, number, c: boolean];",
        "type Foo = [a: string, number, boolean];",
        "type Foo = [a?: string, number?];",
        "type Foo = [string?, b: number];",
        "type Foo = [string, ...rest: number[]];",
        "type Foo = [a: string, ...number[]];",
        "type Foo = [a?: string, number];",
        "type Foo = readonly [a: string, number];",
        "type Foo = [[a: number, number]];",
        "type Foo = [a: string, /* unlabeled */ number];",
    ];

    Tester::new(ConsistentTupleLabels::NAME, ConsistentTupleLabels::PLUGIN, pass, fail)
        .test_and_snapshot();
}
