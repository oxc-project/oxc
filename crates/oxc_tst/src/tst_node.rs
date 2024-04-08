use oxc_ast::ast::*;
use oxc_ast::AstOwnedKind;
use oxc_semantic::AstNodeId;

use crate::tst::TstBuilder;

pub enum TstNodeChildren {
    None,
    One(AstNodeId),
    Many(Vec<AstNodeId>),
}

pub struct TstNode<'a> {
    /// The node itself.
    pub node: AstOwnedKind<'a>,

    /// ID of itself.
    pub id: AstNodeId,

    /// ID of direct parent.
    pub parent_id: AstNodeId,

    /// IDs of all ancestor parents.
    pub parent_ids: Vec<AstNodeId>,

    /// IDs of all children.
    pub children_ids: TstNodeChildren,
}

pub trait IntoTst<'a> {
    fn into_tst(self, builder: &mut TstBuilder<'a>) -> TstNode<'a>;
}

impl<'a> IntoTst<'a> for Program<'a> {
    fn into_tst(mut self, builder: &mut TstBuilder<'a>) -> TstNode<'a> {
        let mut node = builder.create_node();

        builder.push_parent(node.id);

        node.children_ids = TstNodeChildren::Many(
            self.body.drain(..).map(|stmt| builder.map_statement(stmt)).collect(),
        );

        builder.pop_parent();

        node.node = AstOwnedKind::Program(self);
        node
    }
}

impl<'a> IntoTst<'a> for BlockStatement<'a> {
    fn into_tst(mut self, builder: &mut TstBuilder<'a>) -> TstNode<'a> {
        let mut node = builder.create_node();

        builder.push_parent(node.id);

        node.children_ids = TstNodeChildren::Many(
            self.body.drain(..).map(|stmt| builder.map_statement(stmt)).collect(),
        );

        builder.pop_parent();

        node.node = AstOwnedKind::BlockStatement(self);
        node
    }
}

impl<'a> IntoTst<'a> for ExpressionStatement<'a> {
    fn into_tst(mut self, builder: &mut TstBuilder<'a>) -> TstNode<'a> {
        let mut node = builder.create_node();

        node.children_ids = TstNodeChildren::One(
            builder.map_expression(std::mem::replace(&mut self.expression, Expression::None)),
        );

        node.node = AstOwnedKind::ExpressionStatement(self);
        node
    }
}

impl<'a> IntoTst<'a> for NumericLiteral<'a> {
    fn into_tst(self, builder: &mut TstBuilder<'a>) -> TstNode<'a> {
        let mut node = builder.create_node();
        node.node = AstOwnedKind::NumericLiteral(self);
        node
    }
}
