use rustc_hash::FxHashSet;

/// A set of AstKind variants, used for storing the unique node types detected in a rule,
/// or a portion of the rule file.
#[derive(Clone)]
pub struct NodeTypeSet {
    node_types: FxHashSet<String>,
}

impl NodeTypeSet {
    /// Create a new set of node variants
    pub fn new() -> Self {
        Self { node_types: FxHashSet::default() }
    }

    /// Insert a variant into the set
    pub fn insert(&mut self, node_type_variant: String) {
        self.node_types.insert(node_type_variant);
    }

    /// Returns `true` if there are no node types in the set.
    pub fn is_empty(&self) -> bool {
        self.node_types.is_empty()
    }

    /// Extend the set with another set of node types.
    pub fn extend(&mut self, other: NodeTypeSet) {
        self.node_types.extend(other.node_types);
    }

    /// Returns the generated code string to initialize an `AstTypesBitset` with the variants
    /// in this set.
    pub fn to_ast_type_bitset_string(&self) -> String {
        let mut variants: Vec<&str> =
            self.node_types.iter().map(std::string::String::as_str).collect();
        variants.sort_unstable();
        let type_idents: Vec<String> =
            variants.into_iter().map(|v| format!("AstType::{v}")).collect();
        format!("AstTypesBitset::from_types(&[{}])", type_idents.join(", "))
    }
}
