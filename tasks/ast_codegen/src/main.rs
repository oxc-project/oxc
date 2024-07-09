// TODO: remove me please!
#![allow(dead_code, unused_imports)]
mod defs;
mod fmt;
mod generators;
mod linker;
mod schema;
mod util;

use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::PathBuf,
    rc::Rc,
};

use fmt::{cargo_fmt, pprint};
use itertools::Itertools;
use proc_macro2::TokenStream;
use syn::parse_file;

use defs::TypeDef;
use generators::{AstBuilderGenerator, AstGenerator, AstKindGenerator, VisitGenerator};
use linker::{linker, Linker};
use schema::{Inherit, Module, REnum, RStruct, RType, Schema};

use crate::generators::ImplGetSpanGenerator;

type Result<R> = std::result::Result<R, String>;
type TypeId = usize;
type TypeName = String;
type TypeTable = Vec<TypeRef>;
type IdentTable = HashMap<TypeName, TypeId>;
type TypeRef = Rc<RefCell<RType>>;

#[derive(Default)]
struct AstCodegen {
    files: Vec<PathBuf>,
    generators: Vec<Box<dyn Generator>>,
}

trait Generator {
    fn name(&self) -> &'static str;
    fn generate(&mut self, ctx: &CodegenCtx) -> GeneratorOutput;
}

#[derive(Debug, Clone)]
enum GeneratorOutput {
    None,
    One(TokenStream),
    Many(HashMap<String, TokenStream>),
    Info(String),
}

impl GeneratorOutput {
    pub fn as_none(&self) {
        assert!(matches!(self, Self::None));
    }

    pub fn as_one(&self) -> &TokenStream {
        if let Self::One(it) = self {
            it
        } else {
            panic!();
        }
    }

    pub fn as_many(&self) -> &HashMap<String, TokenStream> {
        if let Self::Many(it) = self {
            it
        } else {
            panic!();
        }
    }

    pub fn as_info(&self) -> &String {
        if let Self::Info(it) = self {
            it
        } else {
            panic!();
        }
    }
}

struct CodegenCtx {
    modules: Vec<Module>,
    ty_table: TypeTable,
    ident_table: IdentTable,
}

struct CodegenResult {
    /// One schema per definition file
    schema: Vec<Schema>,
    outputs: Vec<(/* generator name */ &'static str, /* output */ GeneratorOutput)>,
}

impl CodegenCtx {
    fn new(mods: Vec<Module>) -> Self {
        // worst case len
        let len = mods.iter().fold(0, |acc, it| acc + it.items.len());
        let defs = mods.iter().flat_map(|it| it.items.iter());

        let mut ty_table = TypeTable::with_capacity(len);
        let mut ident_table = IdentTable::with_capacity(len);
        for def in defs {
            if let Some(ident) = def.borrow().ident() {
                let ident = ident.to_string();
                let type_id = ty_table.len();
                ty_table.push(TypeRef::clone(def));
                ident_table.insert(ident, type_id);
            }
        }
        Self { modules: mods, ty_table, ident_table }
    }

    fn find(&self, key: &TypeName) -> Option<TypeRef> {
        self.type_id(key).map(|id| TypeRef::clone(&self.ty_table[*id]))
    }

    fn type_id<'b>(&'b self, key: &'b TypeName) -> Option<&'b TypeId> {
        self.ident_table.get(key)
    }
}

impl AstCodegen {
    #[must_use]
    fn add_file<P>(mut self, path: P) -> Self
    where
        P: AsRef<str>,
    {
        self.files.push(path.as_ref().into());
        self
    }

    #[must_use]
    fn with<G>(mut self, generator: G) -> Self
    where
        G: Generator + 'static,
    {
        self.generators.push(Box::new(generator));
        self
    }

    fn generate(self) -> Result<CodegenResult> {
        let modules = self
            .files
            .into_iter()
            .map(Module::from)
            .map(Module::load)
            .map_ok(Module::expand)
            .flatten()
            .map_ok(Module::analyze)
            .collect::<Result<Result<Vec<_>>>>()??;

        let ctx = CodegenCtx::new(modules);
        ctx.link(linker)?;

        let outputs = self
            .generators
            .into_iter()
            .map(|mut gen| (gen.name(), gen.generate(&ctx)))
            .collect_vec();

        let schema = ctx.modules.into_iter().map(Module::build).collect::<Result<Vec<_>>>()?;
        Ok(CodegenResult { schema, outputs })
    }
}

const AST_ROOT_DIR: &str = "crates/oxc_ast";

fn files() -> impl std::iter::Iterator<Item = String> {
    fn path(path: &str) -> String {
        format!("{AST_ROOT_DIR}/src/ast/{path}.rs")
    }

    vec![path("literal"), path("js"), path("ts"), path("jsx")].into_iter()
}

fn output_dir() -> Result<String> {
    let dir = format!("{AST_ROOT_DIR}/src/generated");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

#[allow(clippy::print_stdout)]
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let CodegenResult { outputs, .. } = files()
        .fold(AstCodegen::default(), AstCodegen::add_file)
        .with(AstGenerator)
        .with(AstKindGenerator)
        .with(AstBuilderGenerator)
        .with(ImplGetSpanGenerator)
        .with(VisitGenerator)
        .generate()?;

    let output_dir = output_dir()?;
    let outputs: HashMap<_, _> = outputs.into_iter().collect();

    {
        // write `span.rs` file
        let output = outputs[ImplGetSpanGenerator.name()].as_one();
        let span_content = pprint(output);

        let path = format!("{output_dir}/span.rs");
        let mut file = fs::File::create(path)?;

        file.write_all(span_content.as_bytes())?;
    }

    {
        // write `ast_kind.rs` file
        let output = outputs[AstKindGenerator.name()].as_one();
        let span_content = pprint(output);

        let path = format!("{output_dir}/ast_kind.rs");
        let mut file = fs::File::create(path)?;

        file.write_all(span_content.as_bytes())?;
    }

    {
        // write `ast_builder.rs` file
        let output = outputs[AstBuilderGenerator.name()].as_one();
        let span_content = pprint(output);

        let path = format!("{output_dir}/ast_builder.rs");
        let mut file = fs::File::create(path)?;

        file.write_all(span_content.as_bytes())?;
    }

    {
        // write `visit.rs` and `visit_mut.rs` files
        let output = outputs[VisitGenerator.name()].as_many();
        let content = pprint(&output["visit"]);
        let content_mut = pprint(&output["visit_mut"]);

        let mut visit = fs::File::create(format!("{output_dir}/visit.rs"))?;
        let mut visit_mut = fs::File::create(format!("{output_dir}/visit_mut.rs"))?;

        visit.write_all(content.as_bytes())?;
        visit_mut.write_all(content_mut.as_bytes())?;
    }

    cargo_fmt(".")?;

    // let schema = serde_json::to_string_pretty(&schema).map_err(|e| e.to_string())?;
    // println!("{schema}");
    Ok(())
}
