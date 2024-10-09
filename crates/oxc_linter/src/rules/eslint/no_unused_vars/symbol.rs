use std::{cell::OnceCell, fmt};

use oxc_ast::{
    ast::{
        AssignmentTarget, BindingIdentifier, BindingPattern, IdentifierReference,
        ImportDeclarationSpecifier, VariableDeclarator,
    },
    AstKind,
};
use oxc_semantic::{
    AstNode, AstNodes, NodeId, Reference, ScopeId, ScopeTree, Semantic, SymbolFlags, SymbolId,
    SymbolTable,
};
use oxc_span::{GetSpan, Span};

#[derive(Clone)]
pub(super) struct Symbol<'s, 'a> {
    semantic: &'s Semantic<'a>,
    id: SymbolId,
    flags: SymbolFlags,
    span: OnceCell<Span>,
}

impl PartialEq for Symbol<'_, '_> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

// constructor and simple getters
impl<'s, 'a> Symbol<'s, 'a> {
    pub fn new(semantic: &'s Semantic<'a>, symbol_id: SymbolId) -> Self {
        let flags = semantic.symbols().get_flags(symbol_id);
        Self { semantic, id: symbol_id, flags, span: OnceCell::new() }
    }

    #[inline]
    pub fn id(&self) -> SymbolId {
        self.id
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.symbols().get_name(self.id)
    }

    #[inline]
    pub const fn flags(&self) -> SymbolFlags {
        self.flags
    }

    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.symbols().get_scope_id(self.id)
    }

    #[inline]
    pub fn declaration(&self) -> &AstNode<'a> {
        self.nodes().get_node(self.declaration_id())
    }

    /// Returns `true` if this symbol has any references of any kind. Does not
    /// check if a references is "used" under the criteria of this rule.
    #[inline]
    pub fn has_references(&self) -> bool {
        !self.symbols().get_resolved_reference_ids(self.id).is_empty()
    }

    #[inline]
    pub fn references(&self) -> impl DoubleEndedIterator<Item = &Reference> + '_ {
        self.symbols().get_resolved_references(self.id)
    }

    /// Is this [`Symbol`] declared in the root scope?
    pub fn is_root(&self) -> bool {
        self.symbols().get_scope_id(self.id) == self.scopes().root_scope_id()
    }

    #[inline]
    fn declaration_id(&self) -> NodeId {
        self.symbols().get_declaration(self.id)
    }

    #[inline]
    pub fn nodes(&self) -> &AstNodes<'a> {
        self.semantic.nodes()
    }

    #[inline]
    pub fn scopes(&self) -> &ScopeTree {
        self.semantic.scopes()
    }

    #[inline]
    pub fn symbols(&self) -> &SymbolTable {
        self.semantic.symbols()
    }

    #[inline]
    pub fn iter_parents(&self) -> impl Iterator<Item = &AstNode<'a>> + '_ {
        self.iter_self_and_parents().skip(1)
    }

    pub fn iter_self_and_parents(&self) -> impl Iterator<Item = &AstNode<'a>> + '_ {
        self.nodes().iter_parents(self.declaration_id())
    }

    #[inline]
    pub fn iter_relevant_parents(&self) -> impl Iterator<Item = &AstNode<'a>> + Clone + '_ {
        self.iter_relevant_parents_of(self.declaration_id())
    }

    pub fn iter_relevant_parents_of(
        &self,
        node_id: NodeId,
    ) -> impl Iterator<Item = &AstNode<'a>> + Clone + '_ {
        self.nodes().iter_parents(node_id).skip(1).filter(|n| Self::is_relevant_kind(n.kind()))
    }

    pub fn iter_relevant_parent_and_grandparent_kinds(
        &self,
        node_id: NodeId,
    ) -> impl Iterator<Item = (/* parent */ AstKind<'a>, /* grandparent */ AstKind<'a>)> + Clone + '_
    {
        let parents_iter = self
            .nodes()
            .iter_parents(node_id)
            .map(AstNode::kind)
            // no skip
            .filter(|kind| Self::is_relevant_kind(*kind));

        let grandparents_iter = parents_iter.clone().skip(1);

        parents_iter.zip(grandparents_iter)
    }

    #[inline]
    const fn is_relevant_kind(kind: AstKind<'a>) -> bool {
        !matches!(
            kind,
            AstKind::ParenthesizedExpression(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSInstantiationExpression(_)
                | AstKind::TSNonNullExpression(_)
                | AstKind::TSTypeAssertion(_)
        )
    }

    /// <https://github.com/oxc-project/oxc/issues/4739>
    fn derive_span(&self) -> Span {
        for kind in self.iter_self_and_parents().map(AstNode::kind) {
            match kind {
                AstKind::BindingIdentifier(_) => continue,
                AstKind::BindingRestElement(rest) => return rest.span,
                AstKind::VariableDeclarator(decl) => return self.clean_binding_id(&decl.id),
                AstKind::FormalParameter(param) => return self.clean_binding_id(&param.pattern),
                _ => break,
            }
        }
        self.symbols().get_span(self.id)
    }

    /// <https://github.com/oxc-project/oxc/issues/4739>
    fn clean_binding_id(&self, binding: &BindingPattern) -> Span {
        if binding.kind.is_destructuring_pattern() {
            return self.symbols().get_span(self.id);
        }
        let own = binding.kind.span();
        binding.type_annotation.as_ref().map_or(own, |ann| Span::new(own.start, ann.span.start))
    }
}

impl<'s, 'a> Symbol<'s, 'a> {
    /// Is this [`Symbol`] exported?
    ///
    /// NOTE: does not support CJS right now.
    pub fn is_exported(&self) -> bool {
        let is_in_exportable_scope = self.is_root() || self.is_in_ts_namespace();
        (is_in_exportable_scope
            && (self.flags.contains(SymbolFlags::Export)
                || self.semantic.module_record().exported_bindings.contains_key(self.name())))
            || self.in_export_node()
    }

    #[inline]
    fn is_in_ts_namespace(&self) -> bool {
        self.scopes().get_flags(self.scope_id()).is_ts_module_block()
    }

    /// We need to do this due to limitations of [`Semantic`].
    fn in_export_node(&self) -> bool {
        for parent in self.nodes().iter_parents(self.declaration_id()).skip(1) {
            match parent.kind() {
                AstKind::ModuleDeclaration(module) => {
                    return module.is_export();
                }
                AstKind::ExportDefaultDeclaration(_) => {
                    return true;
                }
                AstKind::VariableDeclaration(_)
                | AstKind::ExpressionArrayElement(_)
                | AstKind::ArrayExpressionElement(_)
                | AstKind::ArrayExpression(_)
                | AstKind::ParenthesizedExpression(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_) => {
                    continue;
                }
                _ => {
                    return false;
                }
            }
        }
        false
    }

    #[inline]
    pub fn is_in_jsx(&self) -> bool {
        self.semantic.source_type().is_jsx()
    }

    #[inline]
    pub fn is_in_ts(&self) -> bool {
        self.semantic.source_type().is_typescript()
    }

    #[inline]
    pub fn get_snippet(&self, span: Span) -> &'a str {
        span.source_text(self.semantic.source_text())
    }
}

impl GetSpan for Symbol<'_, '_> {
    /// [`Span`] for the node declaring the [`Symbol`].
    #[inline]
    fn span(&self) -> Span {
        // TODO: un-comment and replace when BindingIdentifier spans are fixed
        // https://github.com/oxc-project/oxc/issues/4739

        // self.symbols().get_span(self.id)
        *self.span.get_or_init(|| self.derive_span())
    }
}

impl<'a> PartialEq<IdentifierReference<'a>> for Symbol<'_, 'a> {
    fn eq(&self, other: &IdentifierReference<'a>) -> bool {
        // cheap: no resolved reference means its a global reference
        let Some(reference_id) = other.reference_id.get() else {
            return false;
        };
        let reference = self.symbols().get_reference(reference_id);
        reference.symbol_id().is_some_and(|symbol_id| self.id == symbol_id)
    }
}

impl<'a> PartialEq<BindingIdentifier<'a>> for Symbol<'_, 'a> {
    fn eq(&self, id: &BindingIdentifier<'a>) -> bool {
        id.symbol_id.get().is_some_and(|id| self.id == id)
    }
}

impl<'a> PartialEq<VariableDeclarator<'a>> for Symbol<'_, 'a> {
    fn eq(&self, decl: &VariableDeclarator<'a>) -> bool {
        self == &decl.id
    }
}

impl<'a> PartialEq<BindingPattern<'a>> for Symbol<'_, 'a> {
    fn eq(&self, id: &BindingPattern<'a>) -> bool {
        id.get_binding_identifier().is_some_and(|id| self == id)
    }
}

impl<'a> PartialEq<AssignmentTarget<'a>> for Symbol<'_, 'a> {
    fn eq(&self, target: &AssignmentTarget<'a>) -> bool {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(id) => self == id.as_ref(),
            _ => false,
        }
    }
}

impl<'s, 'a> PartialEq<ImportDeclarationSpecifier<'a>> for Symbol<'s, 'a> {
    fn eq(&self, import: &ImportDeclarationSpecifier<'a>) -> bool {
        self == import.local()
    }
}

impl<'s, 'a, T> PartialEq<&T> for Symbol<'s, 'a>
where
    Symbol<'s, 'a>: PartialEq<T>,
{
    fn eq(&self, other: &&T) -> bool {
        self == *other
    }
}

impl fmt::Debug for Symbol<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Symbol")
            .field("id", &self.id)
            .field("name", &self.name())
            .field("flags", &self.flags)
            .field("declaration_node", &self.declaration().kind().debug_name())
            .field("references", &self.references().collect::<Vec<_>>())
            .finish()
    }
}
