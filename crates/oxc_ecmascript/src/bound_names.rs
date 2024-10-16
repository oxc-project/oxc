use oxc_ast::ast::{
    ArrayPattern, AssignmentPattern, BindingIdentifier, BindingPattern, BindingPatternKind,
    BindingRestElement, Class, Declaration, ExportNamedDeclaration, FormalParameter,
    FormalParameters, Function, ImportDeclaration, ImportDeclarationSpecifier, ModuleDeclaration,
    ObjectPattern, VariableDeclaration,
};

/// [`BoundName`](https://tc39.es/ecma262/#sec-static-semantics-boundnames)
pub trait BoundName<'a> {
    fn bound_name<F: FnMut(&BindingIdentifier<'a>)>(&self, f: &mut F);
}

pub trait BoundNames<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier<'a>)>(&self, f: &mut F);
}

impl<'a> BoundNames<'a> for BindingPattern<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier<'a>)>(&self, f: &mut F) {
        match &self.kind {
            BindingPatternKind::BindingIdentifier(ident) => ident.bound_names(f),
            BindingPatternKind::ArrayPattern(array) => array.bound_names(f),
            BindingPatternKind::ObjectPattern(object) => object.bound_names(f),
            BindingPatternKind::AssignmentPattern(assignment) => assignment.bound_names(f),
        }
    }
}

impl<'a> BoundNames<'a> for BindingIdentifier<'a> {
    fn bound_names<F: FnMut(&Self)>(&self, f: &mut F) {
        f(self);
    }
}

impl<'a> BoundNames<'a> for ArrayPattern<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier<'a>)>(&self, f: &mut F) {
        for elem in self.elements.iter().flatten() {
            elem.bound_names(f);
        }
        if let Some(rest) = &self.rest {
            rest.bound_names(f);
        }
    }
}

impl<'a> BoundNames<'a> for ObjectPattern<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier<'a>)>(&self, f: &mut F) {
        for p in &self.properties {
            p.value.bound_names(f);
        }
        if let Some(rest) = &self.rest {
            rest.bound_names(f);
        }
    }
}

impl<'a> BoundNames<'a> for AssignmentPattern<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier<'a>)>(&self, f: &mut F) {
        self.left.bound_names(f);
    }
}

impl<'a> BoundNames<'a> for BindingRestElement<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier<'a>)>(&self, f: &mut F) {
        self.argument.bound_names(f);
    }
}

impl<'a> BoundNames<'a> for FormalParameters<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier<'a>)>(&self, f: &mut F) {
        for item in &self.items {
            item.bound_names(f);
        }
        if let Some(rest) = &self.rest {
            rest.bound_names(f);
        }
    }
}

impl<'a> BoundNames<'a> for Declaration<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier<'a>)>(&self, f: &mut F) {
        match self {
            Declaration::VariableDeclaration(decl) => decl.bound_names(f),
            Declaration::FunctionDeclaration(func) => func.bound_names(f),
            Declaration::ClassDeclaration(decl) => decl.bound_names(f),
            _ => {}
        }
    }
}

impl<'a> BoundNames<'a> for VariableDeclaration<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier<'a>)>(&self, f: &mut F) {
        for declarator in &self.declarations {
            declarator.id.bound_names(f);
        }
    }
}

impl<'a> BoundName<'a> for Function<'a> {
    fn bound_name<F: FnMut(&BindingIdentifier<'a>)>(&self, f: &mut F) {
        if let Some(ident) = &self.id {
            f(ident);
        }
    }
}

impl<'a> BoundNames<'a> for Function<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier<'a>)>(&self, f: &mut F) {
        self.bound_name(f);
    }
}

impl<'a> BoundName<'a> for Class<'a> {
    fn bound_name<F: FnMut(&BindingIdentifier<'a>)>(&self, f: &mut F) {
        if let Some(ident) = &self.id {
            f(ident);
        }
    }
}

impl<'a> BoundNames<'a> for Class<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier<'a>)>(&self, f: &mut F) {
        self.bound_name(f);
    }
}

impl<'a> BoundNames<'a> for FormalParameter<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier<'a>)>(&self, f: &mut F) {
        self.pattern.bound_names(f);
    }
}

impl<'a> BoundNames<'a> for ModuleDeclaration<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier<'a>)>(&self, f: &mut F) {
        match self {
            ModuleDeclaration::ImportDeclaration(decl) => decl.bound_names(f),
            ModuleDeclaration::ExportNamedDeclaration(decl) => decl.bound_names(f),
            _ => {}
        }
    }
}

impl<'a> BoundNames<'a> for ImportDeclaration<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier<'a>)>(&self, f: &mut F) {
        if let Some(specifiers) = &self.specifiers {
            for specifier in specifiers {
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        f(&specifier.local);
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        f(&specifier.local);
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        f(&specifier.local);
                    }
                }
            }
        }
    }
}

impl<'a> BoundNames<'a> for ExportNamedDeclaration<'a> {
    fn bound_names<F: FnMut(&BindingIdentifier<'a>)>(&self, f: &mut F) {
        if let Some(decl) = &self.declaration {
            decl.bound_names(f);
        }
    }
}
