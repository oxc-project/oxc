//! Generator for raw transfer lazy deserializer and visitor.

use std::str;

use oxc_index::{IndexVec, index_vec};

use crate::{
    Generator,
    codegen::{Codegen, DeriveId},
    derives::estree::should_skip_field,
    output::Output,
    schema::{Def, Schema, TypeDef, TypeId, extensions::estree::WalkType},
};

use super::define_generator;

/// Generator for raw transfer lazy deserializer and visitor.
pub struct ESTreeVisitGenerator;

define_generator!(ESTreeVisitGenerator);

impl Generator for ESTreeVisitGenerator {
    fn prepare(&self, schema: &mut Schema, codegen: &Codegen) {
        let estree_derive_id = codegen.get_derive_id_by_name("ESTree");
        WalkTypeCalculator::calculate(estree_derive_id, schema);
    }

    fn generate_many(&self, _schema: &Schema, _codegen: &Codegen) -> Vec<Output> {
        // TODO
        vec![]
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
