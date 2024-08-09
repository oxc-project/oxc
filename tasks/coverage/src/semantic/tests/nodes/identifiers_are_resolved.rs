use crate::{
    semantic::{NodeTestCase, TestName},
    suite::TestResult,
};
use oxc_ast::{ast::BindingIdentifier, AstKind};

/// Tests that all [`AstKind::BindingIdentifier`] and
/// [`AstKind::IdentifierReference`] nodes have their symbols resolved.
#[derive(Debug, Default, Clone)]
pub struct IdentifiersAreResolved;

impl TestName for IdentifiersAreResolved {
    fn name(&self) -> &'static str {
        "nodes/identifiers_are_resolved"
    }
}

impl NodeTestCase for IdentifiersAreResolved {
    fn run_on_node<'a>(
        &self,
        node: &oxc_semantic::AstNode<'a>,
        _semantic: &oxc_semantic::Semantic<'a>,
    ) -> TestResult {
        let pass = match node.kind() {
            AstKind::Function(f) => {
                // allow anonymous functions
                has_bound_identifier(f.id.as_ref())
            }
            AstKind::Class(class) => {
                // allow anonymous classes
                has_bound_identifier(class.id.as_ref())
            }
            AstKind::BindingIdentifier(id) => id.symbol_id.get().is_some(),
            AstKind::TSEnumDeclaration(ts_enum) => ts_enum.id.symbol_id.get().is_some(),
            AstKind::TSInterfaceDeclaration(iface) => iface.id.symbol_id.get().is_some(),
            AstKind::TSTypeAliasDeclaration(alias) => alias.id.symbol_id.get().is_some(),
            #[allow(clippy::match_same_arms)]
            AstKind::TSModuleDeclaration(_) => {
                // NOTE: namespaces do not store their symbol_id. Should they?
                true
            }
            _ => true,
        };

        if pass {
            TestResult::Passed
        } else {
            let name = node.kind().debug_name().to_string();
            TestResult::SemanticError(format!("AST Node {name} has an unbound identifier"))
        }
    }
}

fn has_bound_identifier(id: Option<&BindingIdentifier>) -> bool {
    id.map_or(true, |id| id.symbol_id.get().is_some())
}
