use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{Atom, Span};

use std::{collections::HashSet, mem, ops::DerefMut, rc::Rc};

/// ES2022: Class Static Block
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-class-static-block>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-class-static-block>
pub struct ClassStaticBlock<'a> {
    ast: Rc<AstBuilder<'a>>,
}

impl<'a> ClassStaticBlock<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>) -> Self {
        Self { ast }
    }

    pub fn transform_class_body<'b>(&mut self, class_body: &'b mut ClassBody<'a>) {
        if !class_body.body.iter().any(|e| matches!(e, ClassElement::StaticBlock(..))) {
            return;
        }

        let private_names: HashSet<Atom> = class_body
            .body
            .iter()
            .filter_map(ClassElement::property_key)
            .filter_map(|p| match p {
                PropertyKey::PrivateIdentifier(p) => Some(p.name.clone()),
                _ => None,
            })
            .collect();

        let mut i = 0;
        for element in class_body.body.iter_mut() {
            let ClassElement::StaticBlock(block) = element else {
                continue;
            };

            let static_block_private_id = generate_uid(&private_names, &mut i);
            let key = PropertyKey::PrivateIdentifier(self.ast.alloc(PrivateIdentifier {
                span: Span::default(),
                name: static_block_private_id.clone(),
            }));

            let value = (block.body.len() == 1
                && matches!(block.body[0], Statement::ExpressionStatement(..)))
            .then(|| {
                // We special-case the single expression case to avoid the iife, since it's common.
                let stmt = self.ast.move_statement(&mut block.body.deref_mut()[0]);
                match stmt {
                    Statement::ExpressionStatement(mut expr_stmt) => {
                        self.ast.move_expression(&mut expr_stmt.expression)
                    }
                    _ => unreachable!(),
                }
            })
            .unwrap_or_else(|| {
                let statements = mem::replace(&mut block.body, self.ast.new_vec());
                let callee = self.ast.parenthesized_expression(
                    Span::default(),
                    self.ast.arrow_expression(
                        Span::default(),
                        false,
                        false,
                        false,
                        self.ast.formal_parameters(
                            Span::default(),
                            FormalParameterKind::ArrowFormalParameters,
                            self.ast.new_vec(),
                            None,
                        ),
                        self.ast.function_body(Span::default(), self.ast.new_vec(), statements),
                        None,
                        None,
                    ),
                );

                self.ast.call_expression(Span::default(), callee, self.ast.new_vec(), false, None)
            });

            *element = self.ast.class_property(
                block.span,
                key,
                Some(value),
                false,
                true,
                self.ast.new_vec(),
            );
        }
    }
}

fn generate_uid(deny_list: &HashSet<Atom>, i: &mut u32) -> Atom {
    *i += 1;

    let mut uid: Atom = if *i == 1 { "_".to_string() } else { format!("_{i}") }.into();
    while deny_list.contains(&uid) {
        *i += 1;
        uid = format!("_{i}").into();
    }

    uid
}
