use rustc_hash::FxHashMap;

use crate::{derives::Derive, DERIVES, GENERATORS};

pub type DeriveId = usize;
pub type GeneratorId = usize;

#[derive(Clone, Copy, Debug)]
enum AttrTarget {
    Derive(DeriveId),
    Generator(GeneratorId),
}

impl AttrTarget {
    fn name(self) -> &'static str {
        match self {
            Self::Derive(id) => DERIVES[id].trait_name(),
            Self::Generator(_id) => "Unknown generator", // TODO
        }
    }
}

pub struct Codegen {
    /// Mapping from derive name to `DeriveId`
    derive_name_to_id: FxHashMap<&'static str, DeriveId>,
    /// Mapping from type attr to ID of derive/generator which uses the attr
    #[expect(dead_code)]
    type_attrs: FxHashMap<&'static str, AttrTarget>,
    /// Mapping from struct field attr to ID of derive/generator which uses the attr
    #[expect(dead_code)]
    field_attrs: FxHashMap<&'static str, AttrTarget>,
    /// Mapping from enum variant attr to ID of derive/generator which uses the attr
    #[expect(dead_code)]
    variant_attrs: FxHashMap<&'static str, AttrTarget>,
}

impl Codegen {
    pub fn new() -> Self {
        let mut derive_name_to_id = FxHashMap::default();

        let mut type_attrs = FxHashMap::default();
        let mut field_attrs = FxHashMap::default();
        let mut variant_attrs = FxHashMap::default();

        for (id, &derive) in DERIVES.iter().enumerate() {
            derive_name_to_id.insert(derive.trait_name(), id);

            let target = AttrTarget::Derive(id);
            for &type_attr in derive.type_attrs() {
                let old_target = type_attrs.insert(type_attr, target);
                if let Some(old_target) = old_target {
                    panic!(
                        "Two derives expect same type attr {type_attr:?}: {} and {}",
                        old_target.name(),
                        target.name()
                    );
                }
            }

            for &field_attr in derive.field_attrs() {
                let old_target = field_attrs.insert(field_attr, target);
                if let Some(old_target) = old_target {
                    panic!(
                        "Two derives expect same struct field attr {field_attr:?}: {} and {}",
                        old_target.name(),
                        target.name()
                    );
                }
            }

            for &variant_attr in derive.variant_attrs() {
                let old_target = variant_attrs.insert(variant_attr, target);
                if let Some(old_target) = old_target {
                    panic!(
                        "Two derives expect same enum variant attr {variant_attr:?}: {} and {}",
                        old_target.name(),
                        target.name()
                    );
                }
            }
        }

        for (id, &generator) in GENERATORS.iter().enumerate() {
            let target = AttrTarget::Generator(id);

            for &type_attr in generator.type_attrs() {
                let old_target = type_attrs.insert(type_attr, target);
                if let Some(old_target) = old_target {
                    panic!(
                        "Two derives/generators expect same type attr {type_attr:?}: {} and {}",
                        old_target.name(),
                        target.name()
                    );
                }
            }

            for &field_attr in generator.field_attrs() {
                let old_target = field_attrs.insert(field_attr, target);
                if let Some(old_target) = old_target {
                    panic!(
                        "Two derives/generators expect same struct field attr {field_attr:?}: {} and {}",
                        old_target.name(),
                        target.name()
                    );
                }
            }

            for &variant_attr in generator.variant_attrs() {
                let old_target = variant_attrs.insert(variant_attr, target);
                if let Some(old_target) = old_target {
                    panic!(
                        "Two derives/generators expect same enum variant attr {variant_attr:?}: {} and {}",
                        old_target.name(),
                        target.name()
                    );
                }
            }
        }

        Self { derive_name_to_id, type_attrs, field_attrs, variant_attrs }
    }

    #[expect(clippy::unused_self)]
    pub fn get_derive(&self, id: DeriveId) -> &'static dyn Derive {
        DERIVES[id]
    }

    pub fn get_derive_id_by_name(&self, name: &str) -> DeriveId {
        self.derive_name_to_id.get(name).copied().unwrap_or_else(|| {
            panic!("Unknown derive trait {name:?}");
        })
    }

    #[expect(dead_code)]
    pub fn get_derive_by_name(&self, name: &str) -> &dyn Derive {
        self.get_derive(self.get_derive_id_by_name(name))
    }
}
