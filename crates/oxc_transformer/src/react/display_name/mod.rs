use std::rc::Rc;

use oxc_allocator::Box;
use oxc_ast::ast::*;
use oxc_span::{Atom, SPAN};
use oxc_traverse::{Ancestor, FinderRet, TraverseCtx};

use crate::context::Ctx;

/// [plugin-transform-react-display-name](https://babeljs.io/docs/babel-plugin-transform-react-display-name)
///
/// This plugin is included in `preset-react`.
///
/// ## Example
///
/// In: `var bar = createReactClass({});`
/// Out: `var bar = createReactClass({ displayName: "bar" });`
pub struct ReactDisplayName<'a> {
    ctx: Ctx<'a>,
}

impl<'a> ReactDisplayName<'a> {
    pub fn new(ctx: &Ctx<'a>) -> Self {
        Self { ctx: Rc::clone(ctx) }
    }
}

// Transforms
impl<'a> ReactDisplayName<'a> {
    pub fn transform_call_expression(
        &self,
        call_expr: &mut CallExpression<'a>,
        ctx: &TraverseCtx<'a>,
    ) {
        let Some(obj_expr) = Self::get_object_from_create_class(call_expr) else {
            return;
        };

        let name = ctx.find_ancestor(|ancestor| {
            match ancestor {
                // `foo = React.createClass({})`
                Ancestor::AssignmentExpressionRight(assign_expr) => match &assign_expr.left() {
                    AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                        FinderRet::Found(ident.name.clone())
                    }
                    target => {
                        if let Some(target) = target.as_member_expression() {
                            if let Some(name) = target.static_property_name() {
                                FinderRet::Found(ctx.ast.new_atom(name))
                            } else {
                                FinderRet::Stop
                            }
                        } else {
                            FinderRet::Stop
                        }
                    }
                },
                // `let foo = React.createClass({})`
                Ancestor::VariableDeclaratorInit(declarator) => match &declarator.id().kind {
                    BindingPatternKind::BindingIdentifier(ident) => {
                        FinderRet::Found(ident.name.clone())
                    }
                    _ => FinderRet::Stop,
                },
                // `{foo: React.createClass({})}`
                Ancestor::ObjectPropertyValue(prop) => {
                    if let Some(name) = prop.key().static_name() {
                        FinderRet::Found(ctx.ast.new_atom(&name))
                    } else {
                        FinderRet::Stop
                    }
                }
                // `export default React.createClass({})`
                // Uses the current file name as the display name.
                Ancestor::ExportDefaultDeclarationDeclaration(_) => {
                    FinderRet::Found(ctx.ast.new_atom(&self.ctx.filename))
                }
                // Stop crawling up when hit a statement
                _ if ancestor.is_via_statement() => FinderRet::Stop,
                _ => FinderRet::Continue,
            }
        });

        if let Some(name) = name {
            self.add_display_name(obj_expr, name);
        }
    }
}

impl<'a> ReactDisplayName<'a> {
    /// Get the object from `React.createClass({})` or `createReactClass({})`
    fn get_object_from_create_class<'b>(
        call_expr: &'b mut CallExpression<'a>,
    ) -> Option<&'b mut Box<'a, ObjectExpression<'a>>> {
        if match &call_expr.callee {
            callee @ match_member_expression!(Expression) => {
                !callee.to_member_expression().is_specific_member_access("React", "createClass")
            }
            Expression::Identifier(ident) => ident.name != "createReactClass",
            _ => true,
        } {
            return None;
        }
        // Only 1 argument being the object expression.
        if call_expr.arguments.len() != 1 {
            return None;
        }
        let arg = call_expr.arguments.get_mut(0)?;
        match arg {
            Argument::ObjectExpression(obj_expr) => Some(obj_expr),
            _ => None,
        }
    }

    /// Add key value `displayName: name` to the `React.createClass` object.
    fn add_display_name(&self, obj_expr: &mut ObjectExpression<'a>, name: Atom<'a>) {
        const DISPLAY_NAME: &str = "displayName";
        // Not safe with existing display name.
        let not_safe = obj_expr.properties.iter().any(|prop| {
            matches!(prop, ObjectPropertyKind::ObjectProperty(p) if p.key.static_name().is_some_and(|name| name == DISPLAY_NAME))
        });
        if not_safe {
            return;
        }
        let object_property = {
            let kind = PropertyKind::Init;
            let identifier_name = IdentifierName::new(SPAN, self.ctx.ast.new_atom(DISPLAY_NAME));
            let key = self.ctx.ast.property_key_identifier(identifier_name);
            let string_literal = StringLiteral::new(SPAN, name);
            let value = self.ctx.ast.literal_string_expression(string_literal);
            self.ctx.ast.object_property(SPAN, kind, key, value, None, false, false, false)
        };
        obj_expr.properties.insert(0, ObjectPropertyKind::ObjectProperty(object_property));
    }
}
