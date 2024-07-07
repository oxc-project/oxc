use std::cell::Cell;

#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
#[allow(clippy::wildcard_imports)]
use oxc_ast::{visit::walk::*, Visit};
use oxc_span::{Atom, GetSpan, Span};
use oxc_syntax::scope::{ScopeFlags, ScopeId};
use rustc_hash::FxHashSet;

/// Linear tree of declaration scopes.
pub struct GlobalSymbolBindingTracker {
    depth: u8,
    symbol_binding_depth: Option<u8>,
    global_this_binding_depth: Option<u8>,
    computed_properties_using_non_global_symbol: FxHashSet<Span>,
    computed_properties_using_non_global_global_this: FxHashSet<Span>,
}

impl GlobalSymbolBindingTracker {
    pub fn new() -> Self {
        Self {
            depth: 0,
            symbol_binding_depth: None,
            global_this_binding_depth: None,
            computed_properties_using_non_global_symbol: FxHashSet::default(),
            computed_properties_using_non_global_global_this: FxHashSet::default(),
        }
    }

    fn does_computed_property_reference_non_global_symbol(&self, key: &PropertyKey) -> bool {
        self.computed_properties_using_non_global_symbol.contains(&key.span())
    }

    fn does_computed_property_reference_non_global_global_this(&self, key: &PropertyKey) -> bool {
        self.computed_properties_using_non_global_global_this.contains(&key.span())
    }

    pub fn does_computed_property_reference_well_known_symbol(&self, key: &PropertyKey) -> bool {
        if let PropertyKey::StaticMemberExpression(expr) = key {
            if let Expression::Identifier(identifier) = &expr.object {
                identifier.name == "Symbol"
                    && !self.does_computed_property_reference_non_global_symbol(key)
            } else if let Expression::StaticMemberExpression(static_member) = &expr.object {
                if let Expression::Identifier(identifier) = &static_member.object {
                    identifier.name == "globalThis"
                        && static_member.property.name == "Symbol"
                        && !self.does_computed_property_reference_non_global_global_this(key)
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    fn handle_name_binding(&mut self, name: &Atom) {
        match name.as_str() {
            "Symbol" if self.symbol_binding_depth.is_none() => {
                self.symbol_binding_depth = Some(self.depth);
            }
            "globalThis" if self.global_this_binding_depth.is_none() => {
                self.global_this_binding_depth = Some(self.depth);
            }
            _ => {}
        }
    }
}

impl<'a> Visit<'a> for GlobalSymbolBindingTracker {
    fn enter_scope(&mut self, _: ScopeFlags, _: &Cell<Option<ScopeId>>) {
        self.depth += 1;
    }

    fn leave_scope(&mut self) {
        if self.symbol_binding_depth == Some(self.depth) {
            self.symbol_binding_depth = None;
        }
        if self.global_this_binding_depth == Some(self.depth) {
            self.global_this_binding_depth = None;
        }
        self.depth -= 1;
    }

    fn visit_ts_type(&mut self, _: &TSType<'a>) {
        // Optimization: we don't need to traverse down into types.
    }

    fn visit_binding_pattern(&mut self, pattern: &BindingPattern<'a>) {
        if let BindingPatternKind::BindingIdentifier(ident) = &pattern.kind {
            self.handle_name_binding(&ident.name);
        }
        walk_binding_pattern(self, pattern);
    }

    fn visit_declaration(&mut self, declaration: &Declaration<'a>) {
        match declaration {
            Declaration::VariableDeclaration(_) | Declaration::UsingDeclaration(_) => {
                // handled in BindingPattern
            }
            Declaration::FunctionDeclaration(decl) => {
                if let Some(id) = decl.id.as_ref() {
                    self.handle_name_binding(&id.name);
                }
            }
            Declaration::ClassDeclaration(decl) => {
                if let Some(id) = decl.id.as_ref() {
                    self.handle_name_binding(&id.name);
                }
            }
            Declaration::TSModuleDeclaration(decl) => {
                if let TSModuleDeclarationName::Identifier(ident) = &decl.id {
                    self.handle_name_binding(&ident.name);
                }
            }
            Declaration::TSImportEqualsDeclaration(decl) => {
                self.handle_name_binding(&decl.id.name);
                return;
            }
            Declaration::TSEnumDeclaration(decl) => {
                self.handle_name_binding(&decl.id.name);
                return;
            }
            Declaration::TSTypeAliasDeclaration(_) | Declaration::TSInterfaceDeclaration(_) => {
                return
            }
        }
        walk_declaration(self, declaration);
    }

    fn visit_object_property(&mut self, prop: &ObjectProperty<'a>) {
        if prop.computed {
            if let PropertyKey::StaticMemberExpression(expr) = &prop.key {
                if self.symbol_binding_depth.is_some() {
                    if let Expression::Identifier(identifier) = &expr.object {
                        if identifier.name == "Symbol" {
                            self.computed_properties_using_non_global_symbol.insert(expr.span);
                        }
                    }
                }

                if self.global_this_binding_depth.is_some() {
                    if let Expression::StaticMemberExpression(static_member) = &expr.object {
                        if let Expression::Identifier(identifier) = &static_member.object {
                            if identifier.name == "globalThis"
                                && static_member.property.name == "Symbol"
                            {
                                self.computed_properties_using_non_global_global_this
                                    .insert(expr.span);
                            }
                        }
                    }
                }
            }
        }

        walk_object_property(self, prop);
    }
}
