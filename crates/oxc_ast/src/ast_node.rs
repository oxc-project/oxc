use oxc_syntax::node::AstNodeId;

pub trait AstNode {
    fn ast_node_id(&self) -> Option<AstNodeId>;
}
