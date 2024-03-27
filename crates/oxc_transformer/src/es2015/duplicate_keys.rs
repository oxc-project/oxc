#![allow(clippy::similar_names)]

use std::{collections::HashSet, rc::Rc};

use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{CompactStr, SPAN};

use crate::{context::TransformerCtx, options::TransformTarget};

/// ES2015: Duplicate Keys
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-duplicate-keys>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-duplicate-keys>
pub struct DuplicateKeys<'a> {
    ast: Rc<AstBuilder<'a>>,
}

impl<'a> DuplicateKeys<'a> {
    pub fn new(ctx: TransformerCtx<'a>) -> Option<Self> {
        (ctx.options.target < TransformTarget::ES2015 || ctx.options.duplicate_keys)
            .then_some(Self { ast: ctx.ast })
    }

    pub fn transform_object_expression<'b>(&mut self, obj_expr: &'b mut ObjectExpression<'a>) {
        let mut visited_data: HashSet<CompactStr> = HashSet::new();
        let mut visited_getters: HashSet<CompactStr> = HashSet::new();
        let mut visited_setters: HashSet<CompactStr> = HashSet::new();

        for property in obj_expr.properties.iter_mut() {
            let ObjectPropertyKind::ObjectProperty(obj_property) = property else {
                continue;
            };

            if obj_property.computed {
                continue;
            }

            if let Some(name) = &obj_property.key.static_name() {
                let mut is_duplicate = false;

                match obj_property.kind {
                    PropertyKind::Get => {
                        if visited_data.contains(name) || visited_getters.contains(name) {
                            is_duplicate = true;
                        }
                        visited_getters.insert(name.clone());
                    }
                    PropertyKind::Set => {
                        if visited_data.contains(name) || visited_setters.contains(name) {
                            is_duplicate = true;
                        }
                        visited_setters.insert(name.clone());
                    }
                    PropertyKind::Init => {
                        if visited_data.contains(name)
                            || visited_setters.contains(name)
                            || visited_getters.contains(name)
                        {
                            is_duplicate = true;
                        }
                        visited_data.insert(name.clone());
                    }
                }

                if is_duplicate {
                    obj_property.computed = true;
                    let string_literal = StringLiteral::new(SPAN, self.ast.new_atom(name));
                    let expr = self.ast.literal_string_expression(string_literal);
                    obj_property.key = PropertyKey::Expression(expr);
                }
            }
        }
    }
}
