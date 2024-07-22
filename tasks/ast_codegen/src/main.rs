// TODO: remove me please!
#![allow(dead_code, unused_imports)]
mod defs;
mod fmt;
mod generators;
mod linker;
mod markers;
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

use bpaf::{Bpaf, Parser};
use fmt::{cargo_fmt, pprint};
use itertools::Itertools;
use proc_macro2::TokenStream;
use syn::parse_file;

use defs::TypeDef;
use generators::{AstBuilderGenerator, AstKindGenerator, VisitGenerator, VisitMutGenerator};
use linker::{linker, Linker};
use schema::{Inherit, Module, REnum, RStruct, RType, Schema};
use util::{write_all_to, NormalizeError};

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

type GeneratedStream = (&'static str, TokenStream);

#[derive(Debug, Clone)]
enum GeneratorOutput {
    None,
    Info(String),
    Stream(GeneratedStream),
}

impl GeneratorOutput {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn expect_none(&self) {
        assert!(self.is_none());
    }

    pub fn to_info(&self) -> &String {
        if let Self::Info(it) = self {
            it
        } else {
            panic!();
        }
    }

    pub fn to_stream(&self) -> &GeneratedStream {
        if let Self::Stream(it) = self {
            it
        } else {
            panic!();
        }
    }

    pub fn into_info(self) -> String {
        if let Self::Info(it) = self {
            it
        } else {
            panic!();
        }
    }

    pub fn into_stream(self) -> GeneratedStream {
        if let Self::Stream(it) = self {
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

fn output_dir() -> std::io::Result<String> {
    let dir = format!("{AST_ROOT_DIR}/src/generated");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn write_generated_streams(
    streams: impl IntoIterator<Item = GeneratedStream>,
) -> std::io::Result<()> {
    let output_dir = output_dir()?;
    for (name, stream) in streams {
        let content = pprint(&stream);
        let path = format!("{output_dir}/{name}.rs");
        write_all_to(content.as_bytes(), path)?;
    }
    Ok(())
}

#[derive(Debug, Bpaf)]
pub struct CliOptions {
    /// Runs all generators but won't write anything down.
    #[bpaf(switch)]
    dry_run: bool,
    /// Don't run cargo fmt at the end
    #[bpaf(switch)]
    no_fmt: bool,
    /// Path of output `schema.json`.
    schema: Option<std::path::PathBuf>,
}

#[allow(clippy::print_stdout)]
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cli_options = cli_options().run();

    let CodegenResult { outputs, schema } = files()
        .fold(AstCodegen::default(), AstCodegen::add_file)
        .with(AstKindGenerator)
        .with(AstBuilderGenerator)
        .with(ImplGetSpanGenerator)
        .with(VisitGenerator)
        .with(VisitMutGenerator)
        .generate()?;

    let (streams, _): (Vec<_>, Vec<_>) =
        outputs.into_iter().partition(|it| matches!(it.1, GeneratorOutput::Stream(_)));

    if !cli_options.dry_run {
        write_generated_streams(streams.into_iter().map(|it| it.1.into_stream()))?;
    }

    if !cli_options.no_fmt {
        cargo_fmt();
    }

    if let CliOptions { schema: Some(schema_path), dry_run: false, .. } = cli_options {
        let path = schema_path.to_str().expect("invalid path for schema output.");
        let schema = serde_json::to_string_pretty(&schema).normalize()?;
        write_all_to(schema.as_bytes(), path)?;
    }

    Ok(())
}
