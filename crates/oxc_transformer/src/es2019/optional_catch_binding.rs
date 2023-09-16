use oxc_ast::{ast::*, AstBuilder};
use oxc_span::Span;

use std::rc::Rc;

/// ES2019: Optional Catch Binding
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-optional-catch-binding>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-optional-catch-binding>
pub struct OptionalCatchBinding<'a> {
    ast: Rc<AstBuilder<'a>>,
}

impl<'a> OptionalCatchBinding<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>) -> Self {
        Self { ast }
    }

    pub fn transform_catch_clause<'b>(&mut self, clause: &'b mut CatchClause<'a>) {
        if clause.param.is_some() {
            return;
        }
        let binding_identifier = BindingIdentifier::new(Span::default(), "_unused".into());
        let binding_pattern_kind = self.ast.binding_identifier(binding_identifier);
        let binding_pattern = self.ast.binding_pattern(binding_pattern_kind, None, false);
        clause.param = Some(binding_pattern);
    }
}
