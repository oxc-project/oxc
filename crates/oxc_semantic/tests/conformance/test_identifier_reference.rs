use oxc_ast::{ast::IdentifierReference, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::{NodeId, Reference};
use oxc_span::GetSpan;
use oxc_syntax::reference::ReferenceId;

use super::{ConformanceTest, TestResult};
use crate::Semantic;

/// Tests reflexivity between [`IdentifierReference`] AST nodes and their corresponding
/// [`Reference`]s.
///
/// Performs the following checks:
/// 1. All [`IdentifierReference`]s have been populated with a [`ReferenceId`], even if the
///    referenced symbol could not be resolved.
///
/// 2. When an [`IdentifierReference`] is used to find a [`Reference`] in the symbol table, the AST
///    node id associated with that [`Reference`] should be the [`IdentifierReference`]'s AST node
///    id.
#[derive(Debug, Clone, Default)]
pub struct IdentifierReferenceTest;

/// [`IdentifierReference::reference_id`] returned [`None`].
fn missing_reference_id(reference: &IdentifierReference) -> TestResult {
    OxcDiagnostic::error("After semantic analysis, all IdentifierReferences should have a reference_id, even if a symbol could not be resolved.")
        .with_label(reference.span().label("This reference's reference_id is None"))
        .into()
}

/// The [`NodeId`] of the [`IdentifierReference`] did not match the [`NodeId`] of the
/// [`Reference`].
fn node_id_mismatch(
    identifier_reference_id: NodeId,
    identifier_reference: &IdentifierReference,
    reference_id: ReferenceId,
    reference: &Reference,
) -> TestResult {
    OxcDiagnostic::error(
        "NodeId mismatch between an IdentifierReference and its corresponding Reference",
    )
    .with_label(
        identifier_reference
            .span
            .label(format!("This IdentifierReference's NodeId is {identifier_reference_id:?}")),
    )
    .with_help(format!(
        "The Reference with id {reference_id:?} has a NodeId of {:?}",
        reference.node_id()
    ))
    .into()
}

impl ConformanceTest for IdentifierReferenceTest {
    fn name(&self) -> &'static str {
        "identifier-reference"
    }

    fn run_on_node<'a>(
        &self,
        node: &oxc_semantic::AstNode<'a>,
        semantic: &Semantic<'a>,
    ) -> TestResult {
        let AstKind::IdentifierReference(id) = node.kind() else {
            return TestResult::Pass;
        };
        let Some(reference_id) = id.reference_id.get() else {
            return missing_reference_id(id);
        };

        let reference = semantic.symbols().get_reference(reference_id);
        if reference.node_id() != node.id() {
            return node_id_mismatch(node.id(), id, reference_id, reference);
        }

        TestResult::Pass
    }
}
