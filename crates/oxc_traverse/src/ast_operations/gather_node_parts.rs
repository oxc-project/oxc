//! Gather node parts trait.
//!
//! Ported from: <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts>
//!
//! This trait is used to gather all the parts of a node that are identifiers.

#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_syntax_operations::BoundNames;

use super::to_identifier;

pub fn get_var_name_from_node<'a, N: GatherNodeParts<'a>>(node: &N) -> String {
    let mut name = String::new();
    node.gather(&mut |mut part| {
        if name.is_empty() {
            part = part.trim_start_matches('_');
        } else {
            name.push('$');
        }
        name.push_str(part);
    });

    if name.is_empty() {
        name = "ref".to_string();
    } else {
        name.truncate(20);
    }

    to_identifier(name)
}

pub trait GatherNodeParts<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F);
}

// -------------------- ModuleDeclaration --------------------
impl<'a> GatherNodeParts<'a> for ImportDeclaration<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.source.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ExportAllDeclaration<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.source.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ExportNamedDeclaration<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        if let Some(source) = &self.source {
            source.gather(f);
        } else if let Some(declaration) = &self.declaration {
            declaration.gather(f);
        } else {
            for specifier in &self.specifiers {
                specifier.gather(f);
            }
        }
    }
}

impl<'a> GatherNodeParts<'a> for ExportDefaultDeclaration<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.declaration.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ExportDefaultDeclarationKind<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            ExportDefaultDeclarationKind::FunctionDeclaration(decl) => decl.gather(f),
            ExportDefaultDeclarationKind::ClassDeclaration(decl) => decl.gather(f),
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(_) => {}
            match_expression!(ExportDefaultDeclarationKind) => self.to_expression().gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for ExportSpecifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match &self.local {
            ModuleExportName::IdentifierName(ident) => ident.gather(f),
            ModuleExportName::IdentifierReference(ident) => ident.gather(f),
            ModuleExportName::StringLiteral(lit) => lit.gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for ImportSpecifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.local.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ImportDefaultSpecifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.local.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ImportNamespaceSpecifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.local.gather(f);
    }
}

// -------------------- Declaration --------------------

impl<'a> GatherNodeParts<'a> for Declaration<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            Self::FunctionDeclaration(decl) => decl.gather(f),
            Self::ClassDeclaration(decl) => decl.gather(f),
            _ => (),
        }
    }
}

// -------------------- Function --------------------

impl<'a> GatherNodeParts<'a> for Function<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        if let Some(id) = &self.id {
            id.gather(f);
        }
    }
}

impl<'a> GatherNodeParts<'a> for BindingRestElement<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.argument.gather(f);
    }
}

// -------------------- BindingPattern --------------------

impl<'a> GatherNodeParts<'a> for VariableDeclarator<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.id.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for BindingPattern<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.bound_names(&mut |id| f(id.name.as_str()));
    }
}

impl<'a> GatherNodeParts<'a> for ObjectPattern<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.bound_names(&mut |id| f(id.name.as_str()));
    }
}

impl<'a> GatherNodeParts<'a> for ArrayPattern<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.bound_names(&mut |id| f(id.name.as_str()));
    }
}

impl<'a> GatherNodeParts<'a> for AssignmentPattern<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.bound_names(&mut |id| f(id.name.as_str()));
    }
}

// -------------------- Expression --------------------

impl<'a> GatherNodeParts<'a> for Expression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            match_member_expression!(Self) => self.to_member_expression().gather(f),
            Self::Identifier(ident) => ident.gather(f),
            Self::CallExpression(expr) => expr.gather(f),
            Self::NewExpression(expr) => expr.gather(f),
            Self::ObjectExpression(expr) => expr.gather(f),
            Self::ThisExpression(expr) => expr.gather(f),
            Self::Super(expr) => expr.gather(f),
            Self::ImportExpression(expr) => expr.gather(f),
            Self::YieldExpression(expr) => expr.gather(f),
            Self::AwaitExpression(expr) => expr.gather(f),
            Self::AssignmentExpression(expr) => expr.gather(f),
            Self::FunctionExpression(expr) => expr.gather(f),
            Self::ClassExpression(expr) => expr.gather(f),
            Self::ParenthesizedExpression(expr) => expr.gather(f),
            Self::UnaryExpression(expr) => expr.gather(f),
            Self::UpdateExpression(expr) => expr.gather(f),
            Self::MetaProperty(expr) => expr.gather(f),
            Self::JSXElement(expr) => expr.gather(f),
            Self::JSXFragment(expr) => expr.gather(f),
            Self::StringLiteral(expr) => expr.gather(f),
            Self::NumericLiteral(expr) => expr.gather(f),
            Self::BooleanLiteral(expr) => expr.gather(f),
            Self::BigIntLiteral(expr) => expr.gather(f),
            _ => (),
        }
    }
}

impl<'a> GatherNodeParts<'a> for MemberExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            MemberExpression::ComputedMemberExpression(expr) => {
                expr.object.gather(f);
                expr.expression.gather(f);
            }
            MemberExpression::StaticMemberExpression(expr) => {
                expr.object.gather(f);
                expr.property.gather(f);
            }
            MemberExpression::PrivateFieldExpression(expr) => {
                expr.object.gather(f);
                expr.field.gather(f);
            }
        }
    }
}

impl<'a> GatherNodeParts<'a> for CallExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.callee.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for NewExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.callee.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ObjectExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        for prop in &self.properties {
            prop.gather(f);
        }
    }
}

impl<'a> GatherNodeParts<'a> for ThisExpression {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("this");
    }
}

impl<'a> GatherNodeParts<'a> for Super {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("super");
    }
}

impl<'a> GatherNodeParts<'a> for ImportExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("import");
    }
}

impl<'a> GatherNodeParts<'a> for YieldExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("yield");
        if let Some(argument) = &self.argument {
            argument.gather(f);
        }
    }
}

impl<'a> GatherNodeParts<'a> for AwaitExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("await");
        self.argument.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for AssignmentExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.left.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ParenthesizedExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.expression.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for UnaryExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.argument.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for UpdateExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.argument.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for MetaProperty<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.meta.gather(f);
        self.property.gather(f);
    }
}

// -------------------- AssignmentTarget --------------------
impl<'a> GatherNodeParts<'a> for AssignmentTarget<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            match_simple_assignment_target!(Self) => {
                self.to_simple_assignment_target().gather(f);
            }
            match_assignment_target_pattern!(Self) => {}
        }
    }
}

impl<'a> GatherNodeParts<'a> for SimpleAssignmentTarget<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            Self::AssignmentTargetIdentifier(ident) => ident.gather(f),
            match_member_expression!(Self) => self.to_member_expression().gather(f),
            _ => {}
        }
    }
}

// -------------------- Classes --------------------

impl<'a> GatherNodeParts<'a> for Class<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        if let Some(id) = &self.id {
            id.gather(f);
        }
    }
}

impl<'a> GatherNodeParts<'a> for ClassElement<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            ClassElement::PropertyDefinition(def) => def.gather(f),
            ClassElement::MethodDefinition(def) => def.gather(f),
            ClassElement::AccessorProperty(def) => def.gather(f),
            _ => (),
        }
    }
}

impl<'a> GatherNodeParts<'a> for PropertyDefinition<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.key.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for MethodDefinition<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.key.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for AccessorProperty<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.key.gather(f);
    }
}

// -------------------- Objects --------------------

impl<'a> GatherNodeParts<'a> for ObjectPropertyKind<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            ObjectPropertyKind::ObjectProperty(prop) => prop.gather(f),
            ObjectPropertyKind::SpreadProperty(prop) => prop.gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for ObjectProperty<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.key.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for PropertyKey<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            PropertyKey::StaticIdentifier(ident) => ident.gather(f),
            PropertyKey::PrivateIdentifier(ident) => ident.gather(f),
            match_expression!(Self) => self.to_expression().gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for SpreadElement<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.argument.gather(f);
    }
}

// -------------------- Identifiers --------------------

impl<'a> GatherNodeParts<'a> for BindingIdentifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.name.as_str());
    }
}

impl<'a> GatherNodeParts<'a> for IdentifierReference<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.name.as_str());
    }
}

impl<'a> GatherNodeParts<'a> for IdentifierName<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.name.as_str());
    }
}

impl<'a> GatherNodeParts<'a> for PrivateIdentifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.name.as_str());
    }
}

// -------------------- Literals --------------------

impl<'a> GatherNodeParts<'a> for StringLiteral<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.value.as_str());
    }
}

impl<'a> GatherNodeParts<'a> for NumericLiteral<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.raw);
    }
}

impl<'a> GatherNodeParts<'a> for BooleanLiteral {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        if self.value {
            f("true");
        } else {
            f("false");
        }
    }
}

impl<'a> GatherNodeParts<'a> for BigIntLiteral<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.raw.as_str());
    }
}

// -------------------- JSX --------------------

impl<'a> GatherNodeParts<'a> for JSXElement<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.opening_element.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for JSXFragment<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.opening_fragment.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for JSXOpeningElement<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.name.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for JSXOpeningFragment {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("Fragment");
    }
}

impl<'a> GatherNodeParts<'a> for JSXElementName<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            JSXElementName::Identifier(ident) => ident.gather(f),
            JSXElementName::IdentifierReference(ident) => ident.gather(f),
            JSXElementName::NamespacedName(ns) => ns.gather(f),
            JSXElementName::MemberExpression(expr) => expr.gather(f),
            JSXElementName::ThisExpression(expr) => expr.gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for JSXNamespacedName<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.namespace.gather(f);
        self.property.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for JSXMemberExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.object.gather(f);
        self.property.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for JSXMemberExpressionObject<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            JSXMemberExpressionObject::IdentifierReference(ident) => ident.gather(f),
            JSXMemberExpressionObject::MemberExpression(expr) => expr.gather(f),
            JSXMemberExpressionObject::ThisExpression(expr) => expr.gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for JSXIdentifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.name.as_str());
    }
}
