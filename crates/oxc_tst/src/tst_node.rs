use oxc_ast::ast::*;
use oxc_ast::AstOwnedKind;
use oxc_semantic::AstNodeId;

use crate::tst::Tst;

#[derive(Clone, Debug)]
pub enum TstNodeChildren {
    None,
    One(AstNodeId),
    Many(Vec<AstNodeId>),
}

#[derive(Debug)]
pub struct TstNode<'a> {
    /// The node itself.
    pub node: Option<AstOwnedKind<'a>>,

    /// ID of itself.
    pub id: AstNodeId,

    /// ID of direct parent.
    pub parent_id: AstNodeId,

    /// IDs of all ancestor parents.
    pub parent_ids: Vec<AstNodeId>,

    /// IDs of all children.
    pub children_ids: TstNodeChildren,
}

impl<'a> TstNode<'a> {
    pub fn created(mut self, node: AstOwnedKind<'a>) -> Self {
        self.node = Some(node);
        self
    }
}

pub trait IntoTst<'a> {
    fn into_tst(self, builder: &mut Tst<'a>) -> TstNode<'a>;
}

impl<'a> IntoTst<'a> for Program<'a> {
    fn into_tst(mut self, builder: &mut Tst<'a>) -> TstNode<'a> {
        let mut node = builder.create_node();

        builder.push_parent(node.id);

        node.children_ids = TstNodeChildren::Many(
            self.body.drain(..).map(|stmt| builder.map_statement(stmt)).collect(),
        );

        builder.pop_parent();

        node.created(AstOwnedKind::Program(self))
    }
}

impl<'a> IntoTst<'a> for BlockStatement<'a> {
    fn into_tst(mut self, builder: &mut Tst<'a>) -> TstNode<'a> {
        let mut node = builder.create_node();

        builder.push_parent(node.id);

        node.children_ids = TstNodeChildren::Many(
            self.body.drain(..).map(|stmt| builder.map_statement(stmt)).collect(),
        );

        builder.pop_parent();

        node.created(AstOwnedKind::BlockStatement(self))
    }
}

impl<'a> IntoTst<'a> for ExpressionStatement<'a> {
    fn into_tst(mut self, builder: &mut Tst<'a>) -> TstNode<'a> {
        let mut node = builder.create_node();

        node.children_ids = TstNodeChildren::One(
            builder.map_expression(std::mem::replace(&mut self.expression, Expression::None)),
        );

        node.created(AstOwnedKind::ExpressionStatement(self))
    }
}

impl<'a> IntoTst<'a> for NumericLiteral<'a> {
    fn into_tst(self, builder: &mut Tst<'a>) -> TstNode<'a> {
        builder.create_node().created(AstOwnedKind::NumericLiteral(self))
    }
}
