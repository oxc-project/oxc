use oxc_allocator::Allocator;
use oxc_ast::{
    ast::{BindingIdentifier, LabeledStatement},
    AstBuilder, Visit,
};
use oxc_span::{Atom, CompactStr};
use rustc_hash::FxHashSet;

pub struct TransformerNaming<'a> {
    ast: AstBuilder<'a>,
    bindings: FxHashSet<Atom<'a>>,
    labels: FxHashSet<Atom<'a>>,
}

impl<'a> TransformerNaming<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        let ast = AstBuilder::new(allocator);
        Self { ast, bindings: FxHashSet::default(), labels: FxHashSet::default() }
    }

    pub fn has_binding(&self, name: &str) -> bool {
        self.bindings.contains(name)
    }

    pub fn has_label(&self, name: &str) -> bool {
        self.labels.contains(name)
    }

    // <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts#L495>
    pub fn generate_uid(&mut self, name: &str) -> CompactStr {
        for i in 0.. {
            let name = Self::internal_generate_uid(name, i);
            if self.has_binding(&name) || self.has_label(&name) {
                continue;
            }

            // Add the generated name to the bindings set.
            self.bindings.insert(self.ast.new_atom(name.as_str()));

            return name;
        }
        unreachable!()
    }

    // <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts#L523>
    fn internal_generate_uid(name: &str, i: i32) -> CompactStr {
        CompactStr::from(if i > 1 { format!("_{name}{i}") } else { format!("_{name}") })
    }
}

impl<'a> Visit<'a> for TransformerNaming<'a> {
    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        self.bindings.insert(ident.name.clone());
    }

    fn visit_labeled_statement(&mut self, stmt: &LabeledStatement<'a>) {
        self.labels.insert(stmt.label.name.clone());
    }
}
