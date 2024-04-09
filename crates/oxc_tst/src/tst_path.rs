use oxc_ast::ast::*;
use oxc_ast::AstOwnedKind;
use oxc_semantic::AstNodeId;

use crate::tst::Tst;

#[derive(Clone, Debug)]
pub enum TstChildren {
    None,
    One(AstNodeId),
    Many(Vec<AstNodeId>),
    LeftRight(AstNodeId, AstNodeId),
}

impl TstChildren {
    pub fn get_ids(&self) -> Vec<&AstNodeId> {
        match self {
            TstChildren::None => vec![],
            TstChildren::One(id) => vec![id],
            TstChildren::Many(ids) => Vec::from_iter(ids.iter()),
            TstChildren::LeftRight(lid, rid) => vec![lid, rid],
        }
    }
}

#[derive(Debug)]
pub struct TstPath<'a> {
    /// The node itself.
    pub node: Option<AstOwnedKind<'a>>,

    /// ID of itself.
    pub id: AstNodeId,

    /// ID of direct parent.
    pub parent_id: AstNodeId,

    /// IDs of all ancestor parents.
    pub parent_ids: Vec<AstNodeId>,

    /// IDs of all children.
    pub children_ids: TstChildren,
}

impl<'a> TstPath<'a> {
    pub fn as_node(&self) -> &AstOwnedKind<'a> {
        self.node.as_ref().unwrap()
    }

    pub fn created(mut self, node: AstOwnedKind<'a>) -> Self {
        self.node = Some(node);
        self
    }
}

impl<'a> Clone for TstPath<'a> {
    fn clone(&self) -> Self {
        Self {
            node: None,
            id: self.id,
            parent_id: self.parent_id,
            parent_ids: self.parent_ids.clone(),
            children_ids: self.children_ids.clone(),
        }
    }
}

pub trait IntoTst<'a> {
    fn into_tst(self, builder: &mut Tst<'a>) -> TstPath<'a>;
}

impl<'a> IntoTst<'a> for Program<'a> {
    fn into_tst(mut self, builder: &mut Tst<'a>) -> TstPath<'a> {
        let mut path = builder.create_path();

        builder.push_parent(path.id);

        path.children_ids = TstChildren::Many(
            self.body.drain(..).map(|stmt| builder.map_statement(stmt)).collect(),
        );

        builder.pop_parent();

        path.created(AstOwnedKind::Program(self))
    }
}

impl<'a> IntoTst<'a> for BlockStatement<'a> {
    fn into_tst(mut self, builder: &mut Tst<'a>) -> TstPath<'a> {
        let mut path = builder.create_path();

        builder.push_parent(path.id);

        path.children_ids = TstChildren::Many(
            self.body.drain(..).map(|stmt| builder.map_statement(stmt)).collect(),
        );

        builder.pop_parent();

        path.created(AstOwnedKind::BlockStatement(self))
    }
}

impl<'a> IntoTst<'a> for ExpressionStatement<'a> {
    fn into_tst(mut self, builder: &mut Tst<'a>) -> TstPath<'a> {
        let mut path = builder.create_path();

        path.children_ids = TstChildren::One(
            builder.map_expression(std::mem::replace(&mut self.expression, Expression::None)),
        );

        path.created(AstOwnedKind::ExpressionStatement(self))
    }
}

impl<'a> IntoTst<'a> for NumericLiteral<'a> {
    fn into_tst(self, builder: &mut Tst<'a>) -> TstPath<'a> {
        builder.create_path().created(AstOwnedKind::NumericLiteral(self))
    }
}
