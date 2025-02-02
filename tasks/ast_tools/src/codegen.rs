use rustc_hash::FxHashMap;

use crate::{
    logln,
    parse::attr::{AttrPositions, AttrProcessor},
    Derive, Generator, Output, RawOutput, Result, Schema, DERIVES, GENERATORS,
};

pub type DeriveId = usize;
pub type GeneratorId = usize;

/// [`Codegen`] contains all data relating to the running of the codegen overall.
///
/// [`Schema`] is the source of truth on types, and which generators and derives act upon.
/// [`Codegen`] is the engine which runs the generators and derives.
pub struct Codegen {
    /// Mapping from derive name to `DeriveId`
    derive_name_to_id: FxHashMap<&'static str, DeriveId>,
    /// Mapping from attribute name to ID of derive/generator which uses the attr,
    /// and legal positions for the attribute
    attr_processors: FxHashMap<&'static str, (AttrProcessor, AttrPositions)>,
}

impl Codegen {
    /// Create new [`Codegen`].
    pub fn new() -> Self {
        let mut derive_name_to_id = FxHashMap::default();

        let mut attr_processors = FxHashMap::default();

        for (id, &derive) in DERIVES.iter().enumerate() {
            derive_name_to_id.insert(derive.trait_name(), id);

            let processor = AttrProcessor::Derive(id);
            for &(name, positions) in derive.attrs() {
                let existing = attr_processors.insert(name, (processor, positions));
                if let Some((existing_processor, _)) = existing {
                    panic!(
                        "Two derives expect same attr `#[{name:?}]`: {} and {}",
                        existing_processor.name(),
                        processor.name()
                    );
                }
            }
        }

        for (id, &generator) in GENERATORS.iter().enumerate() {
            let processor = AttrProcessor::Generator(id);

            for &(name, positions) in generator.attrs() {
                let existing_processor = attr_processors.insert(name, (processor, positions));
                if let Some((existing_processor, _)) = existing_processor {
                    panic!(
                        "Two derives/generators expect same attr {name:?}: {} and {}",
                        existing_processor.name(),
                        processor.name()
                    );
                }
            }
        }

        Self { derive_name_to_id, attr_processors }
    }

    /// Get a [`Derive`] by its name.
    pub fn get_derive_id_by_name(&self, name: &str) -> DeriveId {
        self.derive_name_to_id.get(name).copied().unwrap_or_else(|| {
            panic!("Unknown derive trait {name:?}");
        })
    }

    /// Get processor (derive or generator) for an attribute, and legal positions for the attribute
    pub fn attr_processor(&self, attr_name: &str) -> Option<(AttrProcessor, AttrPositions)> {
        self.attr_processors.get(attr_name).copied()
    }
}

/// Runner trait.
///
/// This is the super-trait of [`Derive`] and [`Generator`].
///
/// [`Generator`]: crate::Generator
pub trait Runner {
    fn name(&self) -> &'static str;

    fn file_path(&self) -> &'static str;

    fn run(&self, schema: &Schema, codegen: &Codegen) -> Result<Vec<Output>>;
}

/// Get all runners (generators and derives).
pub fn get_runners() -> Vec<GeneratorOrDerive> {
    GENERATORS
        .iter()
        .map(|&gen| GeneratorOrDerive::Generator(gen))
        .chain(DERIVES.iter().map(|&derive| GeneratorOrDerive::Derive(derive)))
        .collect()
}

/// A `Generator` or a `Derive`.
///
/// Provides a single interface for running either.
#[derive(Clone, Copy)]
pub enum GeneratorOrDerive {
    Generator(&'static (dyn Generator + Sync)),
    Derive(&'static (dyn Derive + Sync)),
}

impl GeneratorOrDerive {
    /// Execute `prepare` method on the [`Generator`] or [`Derive`].
    pub fn prepare(self, schema: &mut Schema) {
        match self {
            Self::Generator(generator) => generator.prepare(schema),
            Self::Derive(derive) => derive.prepare(schema),
        }
    }

    /// Run the [`Generator`] or [`Derive`].
    pub fn run(self, schema: &Schema, codegen: &Codegen) -> Vec<RawOutput> {
        let (runner_path, result) = match self {
            Self::Generator(generator) => {
                logln!("Generate {}... ", generator.name());
                (generator.file_path(), generator.run(schema, codegen))
            }
            Self::Derive(derive) => {
                logln!("Derive {}... ", derive.name());
                (derive.file_path(), derive.run(schema, codegen))
            }
        };
        let runner_outputs = result.unwrap();
        runner_outputs.into_iter().map(|output| output.into_raw(runner_path)).collect()
    }
}
