use std::rc::Rc;

use oxc_ast::ast::{ClassBody, ClassElement, PropertyDefinitionType, Statement};
use oxc_span::SPAN;
use rustc_hash::FxHashSet;

use crate::context::Ctx;

pub struct ClassStaticBlock<'a> {
    ctx: Ctx<'a>,
}

impl<'a> ClassStaticBlock<'a> {
    pub fn new(ctx: &Ctx<'a>) -> Self {
        Self { ctx: Rc::clone(&ctx) }
    }

    pub fn transform_class_body(&mut self, body: &mut ClassBody<'a>) {
        let mut private_names = FxHashSet::default();
        let ast = &self.ctx.ast;

        for stmt in &body.body {
            if stmt.is_private() {
                if let Some(name) = stmt.property_key().and_then(|k| k.private_name()) {
                    private_names.insert(name.as_str().to_owned());
                }
            }
        }

        let mut generate_id = || -> String {
            let mut id = "_".to_owned();
            let mut i = 0;

            while private_names.contains(&id) {
                i += 1;
                id = format!("_{i}");
            }

            private_names.insert(id.clone());

            id
        };

        let stmts = ast.move_vec(&mut body.body).into_iter().filter_map(|stmt| {
            if let ClassElement::StaticBlock(mut block) = stmt {
                let private_id = generate_id();

                // We special-case the single expression case to avoid the iife,
                // since it's common.
                let replacement = if block.body.len() == 1
                    && matches!(&block.body[0], Statement::ExpressionStatement(_))
                {
                    match block.body.remove(0) {
                        Statement::ExpressionStatement(mut expr) => {
                            ast.move_expression(&mut expr.expression)
                        }
                        _ => unreachable!(),
                    }
                } else {
                    ast.build_iife(SPAN, ast.move_vec(&mut block.body))
                };

                Some(ast.class_property(
                    PropertyDefinitionType::PropertyDefinition,
                    SPAN,
                    ast.property_key_private_identifier(ast.private_identifier(SPAN, &private_id)),
                    Some(replacement),
                    false,
                    true,
                    ast.new_vec(),
                ))
            } else {
                Some(stmt)
            }
        });

        body.body = ast.new_vec_from_iter(stmts);
    }
}
