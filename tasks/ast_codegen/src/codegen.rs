use std::{cell::RefCell, collections::HashMap, path::PathBuf};

use itertools::Itertools;

use crate::{
    generators::{Generator, GeneratorOutput},
    passes::Pass,
    rust_ast::{self, AstRef},
    schema::{lower_ast_types, Schema, TypeDef},
    Result, TypeId,
};

#[derive(Default)]
pub struct AstCodegen {
    files: Vec<PathBuf>,
    passes: Vec<Box<dyn Runner<Output = (), Context = EarlyCtx>>>,
    generators: Vec<Box<dyn Runner<Output = GeneratorOutput, Context = LateCtx>>>,
}

pub struct AstCodegenResult {
    pub schema: Schema,
    pub outputs: Vec<(/* generator name */ &'static str, /* output */ GeneratorOutput)>,
}

pub trait Runner {
    type Context;
    type Output;
    fn name(&self) -> &'static str;
    fn run(&mut self, ctx: &Self::Context) -> Result<Self::Output>;
}

pub struct EarlyCtx {
    ty_table: Vec<AstRef>,
    ident_table: HashMap<String, TypeId>,
    mods: RefCell<Vec<rust_ast::Module>>,
}

impl EarlyCtx {
    fn new(mods: Vec<rust_ast::Module>) -> Self {
        // worst case len
        let len = mods.iter().fold(0, |acc, it| acc + it.items.len());
        let adts = mods.iter().flat_map(|it| it.items.iter());

        let mut ty_table = Vec::with_capacity(len);
        let mut ident_table = HashMap::with_capacity(len);
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
        P: Pass + Runner<Output = (), Context = EarlyCtx> + 'static,
    {
        self.passes.push(Box::new(pass));
        self
    }

    #[must_use]
    pub fn gen<G>(mut self, generator: G) -> Self
    where
        G: Generator + Runner<Output = GeneratorOutput, Context = LateCtx> + 'static,
    {
        self.generators.push(Box::new(generator));
        self
    }

    pub fn generate(self) -> Result<AstCodegenResult> {
        let modules = self
            .files
            .into_iter()
            .map(rust_ast::Module::from)
            .map(rust_ast::Module::load)
            .map_ok(rust_ast::Module::expand)
            .map_ok(|it| it.map(rust_ast::Module::analyze))
            .collect::<Result<Result<Result<Vec<_>>>>>()???;

        // early passes
        let ctx = {
            let ctx = EarlyCtx::new(modules);
            _ = self
                .passes
                .into_iter()
                .map(|mut runner| runner.run(&ctx).map(|res| (runner.name(), res)))
                .collect::<Result<Vec<_>>>()?;
            ctx.into_late_ctx()
        };

        let outputs = self
            .generators
            .into_iter()
            .map(|mut runner| runner.run(&ctx).map(|res| (runner.name(), res)))
            .collect::<Result<Vec<_>>>()?;

        Ok(AstCodegenResult { outputs, schema: ctx.schema })
    }
}
