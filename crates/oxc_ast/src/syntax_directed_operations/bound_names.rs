use crate::ast::*;

/// [`BoundName`](https://tc39.es/ecma262/#sec-static-semantics-boundnames)
pub trait BoundName {
    fn bound_name<F: FnMut(&BindingIdentifier)>(&self, f: &mut F);
}

pub trait BoundNames {
    fn bound_names<F: FnMut(&BindingIdentifier)>(&self, f: &mut F);
}

impl<'a> BoundNames for BindingPattern<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier)>(&self, f: &mut F) {
        match &self.kind {
            BindingPatternKind::BindingIdentifier(ident) => ident.bound_names(f),
            BindingPatternKind::ArrayPattern(array) => array.bound_names(f),
            BindingPatternKind::ObjectPattern(object) => object.bound_names(f),
            BindingPatternKind::AssignmentPattern(assignment) => assignment.bound_names(f),
        }
    }
}

impl BoundNames for BindingIdentifier {
    fn bound_names<F: FnMut(&Self)>(&self, f: &mut F) {
        f(self);
    }
}

impl<'a> BoundNames for ArrayPattern<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier)>(&self, f: &mut F) {
        for elem in self.elements.iter().flatten() {
            elem.bound_names(f);
        }
        if let Some(rest) = &self.rest {
            rest.bound_names(f);
        }
    }
}

impl<'a> BoundNames for ObjectPattern<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier)>(&self, f: &mut F) {
        for p in &self.properties {
            p.value.bound_names(f);
        }
        if let Some(rest) = &self.rest {
            rest.bound_names(f);
        }
    }
}

impl<'a> BoundNames for AssignmentPattern<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier)>(&self, f: &mut F) {
        self.left.bound_names(f);
    }
}

impl<'a> BoundNames for RestElement<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier)>(&self, f: &mut F) {
        self.argument.bound_names(f);
    }
}

impl<'a> BoundNames for FormalParameters<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier)>(&self, f: &mut F) {
        for item in &self.items {
            item.bound_names(f);
        }
        if let Some(rest) = &self.rest {
            rest.bound_names(f);
        }
    }
}

impl<'a> BoundNames for Declaration<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier)>(&self, f: &mut F) {
        match self {
            Declaration::VariableDeclaration(decl) => decl.bound_names(f),
            Declaration::FunctionDeclaration(func) => func.bound_names(f),
            Declaration::ClassDeclaration(decl) => decl.bound_names(f),
            _ => {}
        }
    }
}

impl<'a> BoundNames for VariableDeclaration<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier)>(&self, f: &mut F) {
        for declarator in &self.declarations {
            declarator.id.bound_names(f);
        }
    }
}

impl<'a> BoundNames for UsingDeclaration<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier)>(&self, f: &mut F) {
        for declarator in &self.declarations {
            declarator.id.bound_names(f);
        }
    }
}

impl<'a> BoundName for Function<'a> {
    fn bound_name<F: FnMut(&BindingIdentifier)>(&self, f: &mut F) {
        if let Some(ident) = &self.id {
            f(ident);
        }
    }
}

impl<'a> BoundNames for Function<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier)>(&self, f: &mut F) {
        self.bound_name(f);
    }
}

impl<'a> BoundName for Class<'a> {
    fn bound_name<F: FnMut(&BindingIdentifier)>(&self, f: &mut F) {
        if let Some(ident) = &self.id {
            f(ident);
        }
    }
}

impl<'a> BoundNames for Class<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier)>(&self, f: &mut F) {
        self.bound_name(f);
    }
}

impl<'a> BoundNames for FormalParameter<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier)>(&self, f: &mut F) {
        self.pattern.bound_names(f);
    }
}

impl<'a> BoundNames for ModuleDeclaration<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier)>(&self, f: &mut F) {
        if let ModuleDeclaration::ImportDeclaration(decl) = &self {
            decl.bound_names(f);
        }
    }
}

impl<'a> BoundNames for ImportDeclaration<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier)>(&self, f: &mut F) {
        if let Some(specifiers) = &self.specifiers {
            for specifier in specifiers {
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => f(&specifier.local),
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        f(&specifier.local)
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        f(&specifier.local)
                    }
                }
            }
        }
    }
}

impl<'a> BoundNames for ExportNamedDeclaration<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier)>(&self, f: &mut F) {
        if let Some(decl) = &self.declaration {
            decl.bound_names(f);
        }
    }
}
