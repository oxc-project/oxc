use oxc_syntax::symbol::SymbolId;

#[allow(clippy::wildcard_imports)]
use crate::ast::*;
use crate::WithBindingIdentifier;
use oxc_span::Atom;

impl<'a> WithBindingIdentifier<'a> for VariableDeclarator<'a> {
    #[inline]
    fn symbol_id(&self) -> Option<SymbolId> {
        self.id.symbol_id()
    }

    #[inline]
    fn name(&self) -> Option<Atom<'a>> {
        self.id.name()
    }
}

impl<'a> WithBindingIdentifier<'a> for CatchParameter<'a> {
    #[inline]
    fn symbol_id(&self) -> Option<SymbolId> {
        self.pattern.symbol_id()
    }

    #[inline]
    fn name(&self) -> Option<Atom<'a>> {
        self.pattern.name()
    }
}

impl<'a> WithBindingIdentifier<'a> for BindingPattern<'a> {
    #[inline]
    fn symbol_id(&self) -> Option<SymbolId> {
        self.kind.symbol_id()
    }

    #[inline]
    fn name(&self) -> Option<Atom<'a>> {
        self.kind.name()
    }
}

impl<'a> WithBindingIdentifier<'a> for AssignmentPattern<'a> {
    #[inline]
    fn symbol_id(&self) -> Option<SymbolId> {
        self.left.symbol_id()
    }

    #[inline]
    fn name(&self) -> Option<Atom<'a>> {
        self.left.name()
    }
}

impl<'a> WithBindingIdentifier<'a> for BindingProperty<'a> {
    #[inline]
    fn symbol_id(&self) -> Option<SymbolId> {
        self.value.symbol_id()
    }

    #[inline]
    fn name(&self) -> Option<Atom<'a>> {
        self.value.name()
    }
}

impl<'a> WithBindingIdentifier<'a> for BindingRestElement<'a> {
    #[inline]
    fn symbol_id(&self) -> Option<SymbolId> {
        self.argument.symbol_id()
    }

    #[inline]
    fn name(&self) -> Option<Atom<'a>> {
        self.argument.name()
    }
}

impl<'a> WithBindingIdentifier<'a> for Function<'a> {
    #[inline]
    fn symbol_id(&self) -> Option<SymbolId> {
        self.id.as_ref().and_then(|id| id.symbol_id.get())
    }

    #[inline]
    fn name(&self) -> Option<Atom<'a>> {
        self.id.as_ref().map(|id| id.name.clone())
    }
}

impl<'a> WithBindingIdentifier<'a> for FormalParameter<'a> {
    #[inline]
    fn symbol_id(&self) -> Option<SymbolId> {
        self.pattern.symbol_id()
    }

    #[inline]
    fn name(&self) -> Option<Atom<'a>> {
        self.pattern.name()
    }
}

impl<'a> WithBindingIdentifier<'a> for Class<'a> {
    #[inline]
    fn symbol_id(&self) -> Option<SymbolId> {
        self.id.as_ref().and_then(|id| id.symbol_id.get())
    }

    #[inline]
    fn name(&self) -> Option<Atom<'a>> {
        self.id.as_ref().map(|id| id.name.clone())
    }
}

impl<'a> WithBindingIdentifier<'a> for ImportSpecifier<'a> {
    #[inline]
    fn symbol_id(&self) -> Option<SymbolId> {
        self.local.symbol_id.get()
    }

    #[inline]
    fn name(&self) -> Option<Atom<'a>> {
        Some(self.local.name.clone())
    }
}

impl<'a> WithBindingIdentifier<'a> for ImportDefaultSpecifier<'a> {
    #[inline]
    fn symbol_id(&self) -> Option<SymbolId> {
        self.local.symbol_id.get()
    }

    #[inline]
    fn name(&self) -> Option<Atom<'a>> {
        Some(self.local.name.clone())
    }
}

impl<'a> WithBindingIdentifier<'a> for ImportNamespaceSpecifier<'a> {
    #[inline]
    fn symbol_id(&self) -> Option<SymbolId> {
        self.local.symbol_id.get()
    }

    #[inline]
    fn name(&self) -> Option<Atom<'a>> {
        Some(self.local.name.clone())
    }
}

impl<'a> WithBindingIdentifier<'a> for TSEnumDeclaration<'a> {
    #[inline]
    fn symbol_id(&self) -> Option<SymbolId> {
        self.id.symbol_id.get()
    }

    #[inline]
    fn name(&self) -> Option<Atom<'a>> {
        Some(self.id.name.clone())
    }
}

impl<'a> WithBindingIdentifier<'a> for TSTypeParameter<'a> {
    #[inline]
    fn symbol_id(&self) -> Option<SymbolId> {
        self.name.symbol_id.get()
    }

    #[inline]
    fn name(&self) -> Option<Atom<'a>> {
        Some(self.name.name.clone())
    }
}

impl<'a> WithBindingIdentifier<'a> for TSTypeAliasDeclaration<'a> {
    #[inline]
    fn symbol_id(&self) -> Option<SymbolId> {
        self.id.symbol_id.get()
    }

    #[inline]
    fn name(&self) -> Option<Atom<'a>> {
        Some(self.id.name.clone())
    }
}

impl<'a> WithBindingIdentifier<'a> for TSInterfaceDeclaration<'a> {
    #[inline]
    fn symbol_id(&self) -> Option<SymbolId> {
        self.id.symbol_id.get()
    }

    #[inline]
    fn name(&self) -> Option<Atom<'a>> {
        Some(self.id.name.clone())
    }
}

impl<'a> WithBindingIdentifier<'a> for TSImportEqualsDeclaration<'a> {
    #[inline]
    fn symbol_id(&self) -> Option<SymbolId> {
        self.id.symbol_id.get()
    }

    #[inline]
    fn name(&self) -> Option<Atom<'a>> {
        Some(self.id.name.clone())
    }
}
