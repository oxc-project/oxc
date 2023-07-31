use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use super::LintRuleMeta;

pub struct RulePathTrieNode {
    /// Name of module
    name: Ident,
    kind: NodeKind,
}

enum NodeKind {
    /// This node is a leaf node, stores its rule structure name
    LeafNode(Ident),
    /// This node is internal node, stores its children
    InternalNode(Vec<RulePathTrieNode>),
}

impl RulePathTrieNode {
    pub fn leaf_node(mod_name: Ident, struct_name: Ident) -> Self {
        Self { name: mod_name, kind: NodeKind::LeafNode(struct_name) }
    }

    pub fn internal_node(name: Ident) -> Self {
        Self { name, kind: NodeKind::InternalNode(vec![]) }
    }

    // pub use root::{
    //   inner1::Rule1,
    //   inner2::Rule2,
    // };
    pub fn use_stmt(&self, is_root: bool) -> TokenStream {
        let name = &self.name;
        let mut stmts = quote! { #name };
        stmts = match &self.kind {
            NodeKind::LeafNode(struct_name) => {
                quote! { #stmts::#struct_name }
            }
            NodeKind::InternalNode(children) => {
                let child_uses = children.iter().map(|node| node.use_stmt(false));
                quote! {
                  #stmts::{
                    #(#child_uses),*
                  }
                }
            }
        };

        if is_root {
            stmts = quote! {
              pub use #stmts;
            }
        }
        stmts
    }
}

pub struct RulePathTrieBuilder {
    root: RulePathTrieNode,
}

impl RulePathTrieBuilder {
    pub fn new() -> Self {
        Self { root: RulePathTrieNode::internal_node(Ident::new("root", Span::call_site())) }
    }

    pub fn push(&mut self, rule_meta: &LintRuleMeta) {
        let mut cur = &mut self.root;
        let mut segments = rule_meta.path.segments.iter().peekable();
        // Sanity check: We don't expect empty path
        assert!(segments.peek().is_some());

        for segment in segments {
            let name = &segment.ident;
            let NodeKind::InternalNode(children) = &mut cur.kind else { unreachable!() };
            let contains_node = children.iter().any(|node| &node.name == name);
            if !contains_node {
                children.push(RulePathTrieNode::internal_node(name.clone()));
            }
            let child = children.iter_mut().find(|node| &node.name == name).unwrap();
            cur = child;
        }
        // The last path is a leaf node
        *cur = RulePathTrieNode::leaf_node(cur.name.clone(), rule_meta.name.clone());
    }

    pub fn finish(self) -> Vec<RulePathTrieNode> {
        let NodeKind::InternalNode(children) = self.root.kind else { unreachable!() };
        children
    }
}
