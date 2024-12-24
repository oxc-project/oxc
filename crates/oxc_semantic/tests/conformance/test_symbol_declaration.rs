use oxc_ast::{
    ast::{BindingIdentifier, BindingPattern},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};
use oxc_syntax::symbol::SymbolId;

use crate::Semantic;

use super::{ConformanceTest, TestResult};

/// Verifies that symbol binding relationships between the SymbolTable and AST nodes are reflexive.
///
/// What does this mean?
/// 1. [`SymbolTable`] stores the AST node id of the node declaring a symbol.
/// 2. That symbol should _always_ be a declaration-like node containing either a
///    [`BindingIdentifier`] or a [`BindingPattern`].
/// 3. The binding pattern or identifier in that node should be populated (e.g. not [`None`]) and
///    contain the symbol id.
///
/// [`SymbolTable`]: oxc_semantic::SymbolTable
#[derive(Debug, Clone, Default)]
pub struct SymbolDeclarationTest;

/// The binding pattern or identifier contained in the declaration node is [`None`].
///
/// See: [`BindingIdentifier::symbol_id`]
fn bound_to_statement_with_no_binding_identifier(
    symbol_id: SymbolId,
    span: Span,
    statement_kind: &str,
) -> TestResult {
    OxcDiagnostic::error(format!(
        "Symbol {symbol_id:?} got bound to a {statement_kind} with no BindingIdentifier"
    ))
    .with_label(span.label("Symbol was declared here"))
    .into()
}

/// [`BindingIdentifier::symbol_id`] contained [`Some`] value, but it was not the [`SymbolId`] used
/// to find it in the [`SymbolTable`].
fn symbol_declaration_not_in_ast_node(
    expected_id: SymbolId,
    binding: &BindingIdentifier,
) -> TestResult {
    let bound_id = binding.symbol_id.get();
    OxcDiagnostic::error(format!(
        "Expected binding to be bound to {expected_id:?} but it was bound to {bound_id:?}"
    ))
    .with_label(binding.span())
    .into()
}

/// Found a non-destructuring [`BindingPattern`] that did not contain a [`BindingIdentifier`].
fn malformed_binding_pattern(expected_id: SymbolId, pattern: &BindingPattern) -> TestResult {
    OxcDiagnostic::error(format!("BindingPattern for {expected_id:?} is not a destructuring pattern but get_binding_identifier() still returned None"))
        .with_label(pattern.span().label("BindingPattern is here"))
        .into()
}

fn invalid_declaration_node(kind: AstKind) -> TestResult {
    OxcDiagnostic::error(format!("Invalid declaration node kind: {}", kind.debug_name()))
        .with_label(kind.span())
        .into()
}

impl ConformanceTest for SymbolDeclarationTest {
    fn name(&self) -> &'static str {
        "symbol-declaration"
    }

    fn run_on_symbol(
        &self,
        symbol_id: oxc_semantic::SymbolId,
        semantic: &Semantic<'_>,
    ) -> TestResult {
        let declaration_id = semantic.symbols().get_declaration(symbol_id);
        let declaration = semantic.nodes().get_node(declaration_id);
        let span = semantic.symbols().get_span(symbol_id);

        match declaration.kind() {
            AstKind::VariableDeclarator(decl) => check_binding_pattern(symbol_id, &decl.id),
            AstKind::CatchParameter(caught) => check_binding_pattern(symbol_id, &caught.pattern),
            AstKind::Function(func) => match func.id.as_ref() {
                Some(id) => check_binding(symbol_id, id),
                None => bound_to_statement_with_no_binding_identifier(symbol_id, span, "Function"),
            },
            AstKind::Class(class) => match class.id.as_ref() {
                Some(id) => check_binding(symbol_id, id),
                None => bound_to_statement_with_no_binding_identifier(symbol_id, span, "Class"),
            },
            AstKind::BindingRestElement(rest) => check_binding_pattern(symbol_id, &rest.argument),
            AstKind::FormalParameter(param) => check_binding_pattern(symbol_id, &param.pattern),
            AstKind::ImportSpecifier(import) => check_binding(symbol_id, &import.local),
            AstKind::ImportNamespaceSpecifier(import) => check_binding(symbol_id, &import.local),
            AstKind::ImportDefaultSpecifier(import) => check_binding(symbol_id, &import.local),
            // =========================== TYPESCRIPT ===========================
            AstKind::TSImportEqualsDeclaration(import) => check_binding(symbol_id, &import.id),
            AstKind::TSTypeParameter(decl) => check_binding(symbol_id, &decl.name),
            // NOTE: namespaces do not store the symbol id they create. We may want to add this in
            // the future.
            AstKind::TSModuleDeclaration(_decl) => TestResult::Pass,
            AstKind::TSTypeAliasDeclaration(decl) => check_binding(symbol_id, &decl.id),
            AstKind::TSInterfaceDeclaration(decl) => check_binding(symbol_id, &decl.id),
            AstKind::TSEnumDeclaration(decl) => check_binding(symbol_id, &decl.id),
            // NOTE: enum members do not store the symbol id they create. We may want to add this
            // in the future.
            AstKind::TSEnumMember(_member) => TestResult::Pass,
            invalid_kind => invalid_declaration_node(invalid_kind),
        }
    }
}

fn check_binding_pattern(expected_id: SymbolId, binding: &BindingPattern) -> TestResult {
    if binding.kind.is_destructuring_pattern() {
        return TestResult::Pass;
    }

    let Some(id) = binding.kind.get_binding_identifier() else {
        return malformed_binding_pattern(expected_id, binding);
    };

    check_binding(expected_id, id)
}

fn check_binding(expected_id: SymbolId, binding: &BindingIdentifier) -> TestResult {
    if binding.symbol_id.get() == Some(expected_id) {
        TestResult::Pass
    } else {
        symbol_declaration_not_in_ast_node(expected_id, binding)
    }
}
