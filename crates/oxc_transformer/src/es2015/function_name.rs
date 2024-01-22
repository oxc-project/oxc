use std::rc::Rc;

// use lazy_static::lazy_static;
use oxc_ast::{ast::*, AstBuilder};
use oxc_semantic::ScopeId;
use oxc_span::{Atom, Span};
use oxc_syntax::identifier::is_identifier_part;
use oxc_syntax::operator::AssignmentOperator;
// use regex::Regex;

use crate::context::TransformerCtx;
use crate::options::TransformOptions;
use crate::utils::is_valid_identifier;
use crate::TransformTarget;

/// ES2015: Function Name
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-function-name>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-function-name>
pub struct FunctionName<'a> {
    _ast: Rc<AstBuilder<'a>>,
    ctx: TransformerCtx<'a>,
    unicode_escapes: bool,
}

impl<'a> FunctionName<'a> {
    pub fn new(
        ast: Rc<AstBuilder<'a>>,
        ctx: TransformerCtx<'a>,
        options: &TransformOptions,
    ) -> Option<Self> {
        (options.target < TransformTarget::ES2015 || options.function_name).then(|| Self {
            _ast: ast,
            ctx,
            // TODO hook up the plugin
            unicode_escapes: true,
        })
    }

    pub fn transform_assignment_expression(&mut self, expr: &mut AssignmentExpression<'a>) {
        if expr.right.is_function() && matches!(expr.operator, AssignmentOperator::Assign) {
            if let AssignmentTarget::SimpleAssignmentTarget(
                SimpleAssignmentTarget::AssignmentTargetIdentifier(target),
            ) = &expr.left
            {
                if let Some(id) =
                    create_valid_identifier(target.span, target.name.clone(), self.unicode_escapes)
                {
                    let scope_id = self.ctx.symbols().get_scope_id_from_span(&target.span);

                    self.transform_expression(&mut expr.right, id, scope_id);
                }
            }
        }
    }

    pub fn transform_object_expression(&mut self, expr: &mut ObjectExpression<'a>) {
        for property_kind in expr.properties.iter_mut() {
            if let ObjectPropertyKind::ObjectProperty(property) = property_kind {
                if property.computed || !property.value.is_function() {
                    continue;
                }

                let (id, scope_id) = match &property.key {
                    PropertyKey::Identifier(ident) => (
                        create_valid_identifier(
                            ident.span,
                            ident.name.clone(),
                            self.unicode_escapes,
                        ),
                        self.ctx.symbols().get_scope_id_from_span(&ident.span),
                    ),
                    PropertyKey::PrivateIdentifier(ident) => (
                        create_valid_identifier(
                            ident.span,
                            ident.name.clone(),
                            self.unicode_escapes,
                        ),
                        self.ctx.symbols().get_scope_id_from_span(&ident.span),
                    ),
                    PropertyKey::Expression(_) => continue,
                };

                if let Some(id) = id {
                    self.transform_expression(&mut property.value, id, scope_id);
                }
            }
        }
    }

    pub fn transform_variable_declarator(&mut self, decl: &mut VariableDeclarator<'a>) {
        let Some(init) = &mut decl.init else { return };

        if let BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind {
            // Create a new ID instead of cloning to avoid local binding/refs
            if let Some(id) =
                create_valid_identifier(ident.span, ident.name.clone(), self.unicode_escapes)
            {
                let scope_id = match ident.symbol_id.get() {
                    Some(symbol_id) => Some(self.ctx.symbols().get_scope_id(symbol_id)),
                    None => self.ctx.symbols().get_scope_id_from_span(&ident.span),
                };

                self.transform_expression(init, id, scope_id);
            }
        };
    }

    // Internal only
    fn transform_expression(
        &mut self,
        expr: &mut Expression<'a>,
        mut id: BindingIdentifier,
        scope_id: Option<ScopeId>,
    ) {
        // function () {} -> function name() {}
        if let Expression::FunctionExpression(func) = expr {
            let mut count = 0;

            // Check for nested params/vars of the same name
            if let Some(scope_id) = scope_id {
                let scopes = self.ctx.scopes();

                for scope in scopes.descendants(scope_id) {
                    for binding in scopes.get_bindings(scope) {
                        if binding.0 == &id.name {
                            count += 1;
                        }
                    }
                }
            }

            // If we're shadowing, change the name
            if count > 0 {
                id.name = Atom::from(format!("{}{}", id.name, count));
            }

            if func.id.is_none() {
                func.id = Some(id);
            }
        }
    }
}

// https://github.com/babel/babel/blob/main/packages/babel-helper-function-name/src/index.ts
// https://github.com/babel/babel/blob/main/packages/babel-types/src/converters/toBindingIdentifierName.ts#L3
// https://github.com/babel/babel/blob/main/packages/babel-types/src/converters/toIdentifier.ts#L4
#[allow(clippy::unnecessary_wraps)]
fn create_valid_identifier(
    span: Span,
    atom: Atom,
    _unicode_escapes: bool,
) -> Option<BindingIdentifier> {
    // NOTE: this regex fails to compile on Rust
    // lazy_static! {
    //     static ref UNICODE_NAME: Regex = Regex::new(r"(?u)[\u{D800}-\u{DFFF}]").unwrap();
    // }

    // if !unicode_escapes && UNICODE_NAME.is_match(atom.as_str()) {
    //     return None;
    // }

    let id = Atom::from(
        atom.chars().map(|c| if is_identifier_part(c) { c } else { '-' }).collect::<String>(),
    );

    let id = if id == "" {
        Atom::from("_")
    } else if id == "eval" || id == "arguments" || id == "null" || !is_valid_identifier(&id, true) {
        Atom::from(format!("_{id}"))
    } else {
        atom
    };

    Some(BindingIdentifier::new(span, id))
}
