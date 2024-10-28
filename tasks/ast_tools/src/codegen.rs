use std::{cell::RefCell, path::PathBuf};

use itertools::Itertools;
use rustc_hash::{FxBuildHasher, FxHashMap};

use crate::{
    log, log_result,
    output::{Output, RawOutput},
    passes::Pass,
    rust_ast::{self, AstRef},
    schema::{lower_ast_types, Schema, TypeDef},
    Result, TypeId,
};

#[derive(Default)]
pub struct AstCodegen {
    files: Vec<PathBuf>,
    passes: Vec<Box<dyn Runner<Context = EarlyCtx>>>,
    generators: Vec<Box<dyn Runner<Context = LateCtx>>>,
}

pub struct AstCodegenResult {
    pub schema: Schema,
    pub outputs: Vec<RawOutput>,
}

pub trait Runner {
    type Context;
    fn verb(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn file_path(&self) -> &'static str;
    fn run(&mut self, ctx: &Self::Context) -> Result<Vec<Output>>;
}

pub struct EarlyCtx {
    ty_table: Vec<AstRef>,
    ident_table: FxHashMap<String, TypeId>,
    mods: RefCell<Vec<rust_ast::Module>>,
}

impl EarlyCtx {
    fn new(mods: Vec<rust_ast::Module>) -> Self {
        // worst case len
        let len = mods.iter().fold(0, |acc, it| acc + it.items.len());
        let adts = mods.iter().flat_map(|it| it.items.iter());

        let mut ty_table = Vec::with_capacity(len);
        let mut ident_table = FxHashMap::with_capacity_and_hasher(len, FxBuildHasher);
        for adt in adts {
            if let Some(ident) = adt.borrow().ident() {
                let ident = ident.to_string();
                let type_id = ty_table.len();
                ty_table.push(AstRef::clone(adt));
                ident_table.insert(ident, type_id);
            }
        }

        Self { ty_table, ident_table, mods: RefCell::new(mods) }
    }

    pub fn chronological_idents(&self) -> impl Iterator<Item = &String> {
        self.ident_table.iter().sorted_by_key(|it| it.1).map(|it| it.0)
    }

    pub fn mods(&self) -> &RefCell<Vec<rust_ast::Module>> {
        &self.mods
    }

    pub fn find(&self, key: &String) -> Option<AstRef> {
        self.type_id(key).map(|id| AstRef::clone(&self.ty_table[id]))
    }

    pub fn type_id(&self, key: &String) -> Option<TypeId> {
        self.ident_table.get(key).copied()
    }

    pub fn ast_ref(&self, id: TypeId) -> AstRef {
        AstRef::clone(&self.ty_table[id])
    }

    fn into_late_ctx(self) -> LateCtx {
        let schema = lower_ast_types(&self);

        LateCtx { schema }
    }
}

pub struct LateCtx {
    schema: Schema,
}

impl LateCtx {
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn type_def(&self, id: TypeId) -> Option<&TypeDef> {
        self.schema.get(id)
    }
}

impl AstCodegen {
    #[must_use]
    pub fn add_file<P>(mut self, path: P) -> Self
    where
        P: AsRef<str>,
    {
        self.files.push(path.as_ref().into());
        self
    }

    #[must_use]
    pub fn pass<P>(mut self, pass: P) -> Self
    where
        P: Pass + Runner<Context = EarlyCtx> + 'static,
    {
        self.passes.push(Box::new(pass));
        self
    }

    #[must_use]
    pub fn generate<G>(mut self, generator: G) -> Self
    where
        G: Runner<Context = LateCtx> + 'static,
    {
        self.generators.push(Box::new(generator));
        self
    }

    pub fn run(mut self) -> Result<AstCodegenResult> {
        let modules = self
            .files
            .into_iter()
            .map(rust_ast::Module::from)
            .map(rust_ast::Module::load)
            .map_ok(rust_ast::Module::expand)
            .map_ok(|it| it.map(rust_ast::Module::analyze))
            .collect::<Result<Result<Result<Vec<_>>>>>()???;

        // Early passes
        let early_ctx = EarlyCtx::new(modules);
        let mut outputs = run_passes(&mut self.passes, &early_ctx)?;

        // Late passes
        let late_ctx = early_ctx.into_late_ctx();
        outputs.extend(run_passes(&mut self.generators, &late_ctx)?);

        Ok(AstCodegenResult { outputs, schema: late_ctx.schema })
    }
}

fn run_passes<C>(runners: &mut [Box<dyn Runner<Context = C>>], ctx: &C) -> Result<Vec<RawOutput>> {
    let mut outputs = vec![];
    for runner in runners {
        log!("{} {}... ", runner.verb(), runner.name());

        let result = runner.run(ctx);
        log_result!(result);
        let runner_outputs = result?;

        let generator_path = runner.file_path();
        outputs.extend(runner_outputs.into_iter().map(|output| output.into_raw(generator_path)));
    }
    Ok(outputs)
}
