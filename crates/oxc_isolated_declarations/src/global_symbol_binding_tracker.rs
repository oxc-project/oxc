use std::cell::Cell;

#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
#[allow(clippy::wildcard_imports)]
use oxc_ast::{visit::walk::*, Visit};
use oxc_span::{Atom, GetSpan, Span};
use oxc_syntax::scope::{ScopeFlags, ScopeId};
use rustc_hash::FxHashSet;

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

    fn visit_statement(&mut self, statement: &Statement<'a>) {
        // Optimizations: Only try to visit parts of statements containing other statements.
        match statement {
            Statement::TSEnumDeclaration(_)
            | Statement::TSTypeAliasDeclaration(_)
            | Statement::TSInterfaceDeclaration(_) => (),
            Statement::WhileStatement(stmt) => walk_statement(self, &stmt.body),
            Statement::DoWhileStatement(stmt) => walk_statement(self, &stmt.body),
            Statement::ForStatement(stmt) => walk_statement(self, &stmt.body),
            Statement::ForInStatement(stmt) => walk_statement(self, &stmt.body),
            Statement::ForOfStatement(stmt) => walk_statement(self, &stmt.body),
            Statement::IfStatement(stmt) => {
                walk_statement(self, &stmt.consequent);
                if let Some(alt) = &stmt.alternate {
                    walk_statement(self, alt);
                }
            }
            Statement::SwitchStatement(stmt) => {
                walk_switch_cases(self, &stmt.cases);
            }
            _ => walk_statement(self, statement),
        }
    }

    fn visit_binding_pattern(&mut self, pattern: &BindingPattern<'a>) {
        if let BindingPatternKind::BindingIdentifier(ident) = &pattern.kind {
            self.handle_name_binding(&ident.name);
        }
        walk_binding_pattern(self, pattern);
    }

    fn visit_assignment_pattern(&mut self, pattern: &AssignmentPattern<'a>) {
        // If the left side of the assignment already has a type annotation, we don't need to visit the right side.
        if pattern.left.type_annotation.is_some() {
            self.visit_binding_pattern(&pattern.left);
            return;
        }
        walk_assignment_pattern(self, pattern);
    }

    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        // If the variable already has a type annotation, we don't need to visit the initializer.
        if decl.id.type_annotation.is_some() {
            self.visit_binding_pattern(&decl.id);
        } else {
            walk_variable_declarator(self, decl);
        }
    }

    fn visit_assignment_expression(&mut self, expr: &AssignmentExpression<'a>) {
        // If the left side of the assignment is a member expression, it won't affect emitted declarations.
        if expr.left.is_member_expression() {
            return;
        }
        walk_assignment_expression(self, expr);
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        // Async and generator functions always need explicit declarations.
        if func.generator || func.r#async {
            return;
        }

        if let Some(id) = func.id.as_ref() {
            self.handle_name_binding(&id.name);
        }

        // If the function already has a return type annotation, we don't need to visit the body.
        if func.return_type.is_some() {
            return;
        }

        walk_function(self, func, flags);
    }

    fn visit_arrow_function_expression(&mut self, func: &ArrowFunctionExpression<'a>) {
        // If the arrow function already has a return type annotation, we don't need to visit the body.
        if func.return_type.is_some() {
            return;
        }

        walk_arrow_function_expression(self, func);
    }

    fn visit_expression(&mut self, expr: &Expression<'a>) {
        match expr {
            Expression::ArrowFunctionExpression(_)
            | Expression::Identifier(_)
            | Expression::ArrayExpression(_)
            | Expression::AssignmentExpression(_)
            | Expression::ClassExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ObjectExpression(_)
            | Expression::ParenthesizedExpression(_) => {
                // Expressions whose types can be inferred, but excluding trivial ones
                // whose types will never contain a Symbol computed property.
                walk_expression(self, expr);
            }
            _ => (),
        }
    }

    fn visit_declaration(&mut self, declaration: &Declaration<'a>) {
        match declaration {
            Declaration::VariableDeclaration(_) | Declaration::FunctionDeclaration(_) => {
                // handled in BindingPattern and Function
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
                return;
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
