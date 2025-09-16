//! Generator for raw transfer lazy deserializer and visitor.

use std::{borrow::Cow, collections::hash_map::Entry, str};

use rustc_hash::FxHashMap;

use oxc_index::{IndexVec, define_index_type, index_vec};

use crate::{
    Generator, NAPI_PARSER_PACKAGE_PATH,
    codegen::{Codegen, DeriveId},
    derives::estree::{get_fieldless_variant_value, should_skip_field},
    output::Output,
    schema::{Def, Schema, StructDef, StructOrEnum, TypeDef, TypeId, extensions::estree::WalkType},
    utils::write_it,
};

use super::define_generator;

define_index_type! {
    /// ID of type in the AST
    pub struct NodeId = u32;
}

/// Generator for raw transfer lazy deserializer and visitor.
pub struct ESTreeVisitGenerator;

define_generator!(ESTreeVisitGenerator);

impl Generator for ESTreeVisitGenerator {
    fn prepare(&self, schema: &mut Schema, codegen: &Codegen) {
        let estree_derive_id = codegen.get_derive_id_by_name("ESTree");
        WalkTypeCalculator::calculate(estree_derive_id, schema);
    }

    fn generate_many(&self, schema: &Schema, _codegen: &Codegen) -> Vec<Output> {
        let types = generate(schema);

        vec![Output::Javascript {
            path: format!("{NAPI_PARSER_PACKAGE_PATH}/generated/visit/types.mjs"),
            code: types,
        }]
    }
}

/// Structure for calculating which types in AST are walked.
struct WalkTypeCalculator<'s> {
    /// `Some(true)` = is walked.
    /// `Some(false)` = is not walked.
    /// `None` = not calculated yet.
    is_walked_states: IndexVec<TypeId, Option<bool>>,
    estree_derive_id: DeriveId,
    schema: &'s mut Schema,
}

impl<'s> WalkTypeCalculator<'s> {
    /// Calculate whether types are visited in ESTree AST.
    ///
    /// Performs a recursive walk through the dependencies of types, starting from `Program`.
    /// For each type, determine if:
    ///
    /// 1. The type is visited or not.
    /// 2. If it is visited, whether the type is:
    ///    a. A normal node, which has children e.g. `BinaryExpression`, or
    ///    b. A leaf node, which doesn't e.g. `IdentifierReference`, `StringLiteral`.
    ///
    /// Criteria for being visited:
    ///
    /// * Structs are visited if either:
    ///   1. It is an AST node i.e. has a `type` field. or
    ///   2. One of its fields is visited.
    /// * Enums are visited if any variant is visited.
    /// * Primitives are never visited.
    /// * Container types (`Box`, `Vec`, `Cell`) are visited if their contained type is visited.
    ///
    /// Types marked `#[ast]` which are not part of ESTree AST are *not* considered as visited.
    ///
    /// Sets `estree.walk_type` for structs, and `estree.is_walked` for enums.
    fn calculate(estree_derive_id: DeriveId, schema: &'s mut Schema) {
        let program_type_id = schema.type_names["Program"];

        let is_walked_states = index_vec![None; schema.types.len()];

        let mut calculator = Self { is_walked_states, estree_derive_id, schema };
        calculator.is_walked(program_type_id);
    }

    fn is_walked(&mut self, type_id: TypeId) -> bool {
        if let Some(is_walked) = self.is_walked_states[type_id] {
            return is_walked;
        }

        let is_walked = match &self.schema.types[type_id] {
            TypeDef::Struct(_) => self.struct_is_walked(type_id),
            TypeDef::Enum(_) => self.enum_is_walked(type_id),
            TypeDef::Primitive(_) => false,
            TypeDef::Option(option_def) => self.is_walked(option_def.inner_type_id),
            TypeDef::Box(box_def) => self.is_walked(box_def.inner_type_id),
            TypeDef::Vec(vec_def) => self.is_walked(vec_def.inner_type_id),
            TypeDef::Cell(cell_def) => self.is_walked(cell_def.inner_type_id),
            TypeDef::Pointer(pointer_def) => self.is_walked(pointer_def.inner_type_id),
        };

        self.is_walked_states[type_id] = Some(is_walked);

        is_walked
    }

    fn struct_is_walked(&mut self, type_id: TypeId) -> bool {
        let struct_def = self.schema.struct_def(type_id);
        if !struct_def.generates_derive(self.estree_derive_id) || struct_def.estree.skip {
            return false;
        }

        let mut has_type_field = !struct_def.estree.no_type;
        if has_type_field {
            // Set state early to avoid infinite loop
            self.is_walked_states[type_id] = Some(true);
        }

        let mut has_walked_field = false;
        for field_index in struct_def.field_indices() {
            let field = &self.schema.struct_def(type_id).fields[field_index];
            if field.name() == "type" {
                if !has_type_field {
                    has_type_field = true;
                    self.is_walked_states[type_id] = Some(true);
                }
            } else if !should_skip_field(field, self.schema) {
                has_walked_field |= self.is_walked(field.type_id);
            }
        }

        let walk_type = match (has_type_field, has_walked_field) {
            (true, true) => WalkType::Node,
            (true, false) => WalkType::Leaf,
            (false, true) => WalkType::Walk,
            (false, false) => WalkType::NoWalk,
        };
        self.schema.struct_def_mut(type_id).estree.walk_type = walk_type;

        walk_type != WalkType::NoWalk
    }

    fn enum_is_walked(&mut self, type_id: TypeId) -> bool {
        let enum_def = self.schema.enum_def(type_id);
        if !enum_def.generates_derive(self.estree_derive_id) || enum_def.estree.skip {
            return false;
        }

        let mut is_walked = false;
        let variant_type_ids =
            enum_def.all_variants(self.schema).filter_map(|v| v.field_type_id).collect::<Vec<_>>();
        for variant_type_id in variant_type_ids {
            is_walked |= self.is_walked(variant_type_id);
        }

        self.schema.enum_def_mut(type_id).estree.is_walked = is_walked;

        is_walked
    }
}

struct ESTreeNode<'s> {
    name: Cow<'s, str>,
    struct_defs: Vec<&'s StructDef>,
    is_leaf: bool,
}

fn generate(schema: &Schema) -> String {
    // Compile hashmap mapping ESTree type name to struct(s) which represent that node type.
    // Can be multiple structs for a single ESTree type.
    // e.g. Rust types `IdentifierName`, `IdentifierReference`, and 3 other types all become ESTree type `Identifier`.
    let mut estree_nodes = FxHashMap::<_, (Vec<&StructDef>, bool)>::default();

    let mut add_estree_node = |name, struct_def| match estree_nodes.entry(name) {
        Entry::Occupied(entry) => {
            let entry = entry.into_mut();
            entry.0.push(struct_def);
            if struct_def.estree.walk_type == WalkType::Node {
                entry.1 = false;
            }
        }
        Entry::Vacant(entry) => {
            entry.insert((vec![struct_def], struct_def.estree.walk_type == WalkType::Leaf));
        }
    };

    'types: for type_def in schema.structs_and_enums() {
        let StructOrEnum::Struct(struct_def) = type_def else {
            continue;
        };
        if matches!(struct_def.estree.walk_type, WalkType::NoWalk | WalkType::Walk) {
            continue;
        }

        for field in &struct_def.fields {
            if field.name() == "type" {
                let field_enum = field.type_def(schema).as_enum().unwrap();
                for variant in &field_enum.variants {
                    assert!(variant.is_fieldless());

                    let name = get_fieldless_variant_value(field_enum, variant);
                    add_estree_node(name, struct_def);
                }
                continue 'types;
            }
        }

        if !struct_def.estree.no_type {
            let name = struct_def.estree.rename.as_deref().unwrap_or_else(|| struct_def.name());
            add_estree_node(Cow::Borrowed(name), struct_def);
        }
    }

    let mut estree_nodes = estree_nodes
        .into_iter()
        .map(|(name, (struct_defs, is_leaf))| ESTreeNode { name, struct_defs, is_leaf })
        .collect::<IndexVec<NodeId, _>>();
    estree_nodes.sort_by(|t1, t2| match t1.is_leaf.cmp(&t2.is_leaf) {
        std::cmp::Ordering::Equal => t1.name.cmp(&t2.name),
        ord => ord.reverse(),
    });

    let mut types_code = "
        // Mapping from node type name to node type ID
        export const NODE_TYPE_IDS_MAP = new Map([
            // Leaf nodes"
        .to_string();

    let mut leaf_nodes_count = None;
    for (node_id, estree_node) in estree_nodes.iter_mut_enumerated() {
        let node_id = node_id.raw();
        if !estree_node.is_leaf && leaf_nodes_count.is_none() {
            leaf_nodes_count = Some(node_id);
            types_code.push_str("\n// Non-leaf nodes");
        }

        let name = &estree_node.name;
        write_it!(types_code, "\n['{name}', {node_id}],");
    }

    let node_types_count = estree_nodes.len();
    let leaf_nodes_count = leaf_nodes_count.unwrap();

    #[rustfmt::skip]
    write_it!(types_code, "
        ]);

        export const NODE_TYPES_COUNT = {node_types_count};
        export const LEAF_NODE_TYPES_COUNT = {leaf_nodes_count};
    ");

    // TODO

    types_code
}

/// Get whether a type is walked in ESTree AST.
fn is_walked(type_def: &TypeDef, schema: &Schema) -> bool {
    match type_def {
        TypeDef::Struct(struct_def) => struct_def.estree.walk_type != WalkType::NoWalk,
        TypeDef::Enum(enum_def) => enum_def.estree.is_walked,
        TypeDef::Primitive(_) => false,
        TypeDef::Option(option_def) => is_walked(option_def.inner_type(schema), schema),
        TypeDef::Box(box_def) => is_walked(box_def.inner_type(schema), schema),
        TypeDef::Vec(vec_def) => is_walked(vec_def.inner_type(schema), schema),
        TypeDef::Cell(cell_def) => is_walked(cell_def.inner_type(schema), schema),
        TypeDef::Pointer(pointer_def) => is_walked(pointer_def.inner_type(schema), schema),
    }
}
