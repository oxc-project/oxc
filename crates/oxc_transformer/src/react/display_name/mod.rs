use std::rc::Rc;

use oxc_allocator::Box;
use oxc_ast::ast::*;
use oxc_span::{Atom, SPAN};

use crate::context::Ctx;

/// [plugin-transform-react-display-name](https://babeljs.io/docs/babel-plugin-transform-react-display-name)
///
/// This plugin is included in `preset-react`.
///
/// ## Example
///
/// In: `var bar = createReactClass({});`
/// Out: `var bar = createReactClass({ displayName: "bar" });`
///
/// NOTE: The current implementation uses the top-down approach on `AssignmentExpression`, `VariableDeclaration`,
/// but can be rewritten with a bottom-up approach.
/// See <https://github.com/babel/babel/blob/08b0472069cd207f043dd40a4d157addfdd36011/packages/babel-plugin-transform-react-display-name/src/index.ts#L88-L98>
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
    /// `foo = React.createClass({})`
    pub fn transform_assignment_expression(&self, assign_expr: &mut AssignmentExpression<'a>) {
        let Some(obj_expr) = Self::get_object_from_create_class(&mut assign_expr.right) else {
            return;
        };
        let name = match &assign_expr.left {
            AssignmentTarget::AssignmentTargetIdentifier(ident) => ident.name.clone(),
            target => {
                if let Some(target) = target.as_member_expression() {
                    if let Some(name) = target.static_property_name() {
                        self.ctx.ast.new_atom(name)
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            }
        };
        self.add_display_name(obj_expr, name);
    }

    /// `let foo = React.createClass({})`
    pub fn transform_variable_declarator(&self, declarator: &mut VariableDeclarator<'a>) {
        let Some(init_expr) = declarator.init.as_mut() else { return };
        let Some(obj_expr) = Self::get_object_from_create_class(init_expr) else {
            return;
        };
        let name = match &declarator.id.kind {
            BindingPatternKind::BindingIdentifier(ident) => ident.name.clone(),
            _ => return,
        };
        self.add_display_name(obj_expr, name);
    }

    /// `{foo: React.createClass({})}`
    pub fn transform_object_property(&self, prop: &mut ObjectProperty<'a>) {
        let Some(obj_expr) = Self::get_object_from_create_class(&mut prop.value) else { return };
        let Some(name) = prop.key.static_name() else { return };
        let name = self.ctx.ast.new_atom(&name);
        self.add_display_name(obj_expr, name);
    }

    /// `export default React.createClass({})`
    /// Uses the current file name as the display name.
    pub fn transform_export_default_declaration(&self, decl: &mut ExportDefaultDeclaration<'a>) {
        let Some(expr) = decl.declaration.as_expression_mut() else { return };
        let Some(obj_expr) = Self::get_object_from_create_class(expr) else { return };
        let name = self.ctx.ast.new_atom(&self.ctx.filename);
        self.add_display_name(obj_expr, name);
    }
}

impl<'a> ReactDisplayName<'a> {
    /// Get the object from `React.createClass({})` or `createReactClass({})`
    fn get_object_from_create_class<'b>(
        e: &'b mut Expression<'a>,
    ) -> Option<&'b mut Box<'a, ObjectExpression<'a>>> {
        let Expression::CallExpression(call_expr) = e else { return None };
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
