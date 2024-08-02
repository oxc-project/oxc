const AST_CRATE: &str = "crates/oxc_ast";

mod defs;
mod fmt;
mod generators;
mod markers;
mod passes;
mod schema;
mod util;

use std::{cell::RefCell, collections::HashMap, io::Read, path::PathBuf, rc::Rc};

use bpaf::{Bpaf, Parser};
use fmt::{cargo_fmt, pprint};
use itertools::Itertools;
use passes::{BuildSchema, Linker, Pass};
use proc_macro2::TokenStream;
use syn::parse_file;

use defs::TypeDef;
use generators::{
    AstBuilderGenerator, AstKindGenerator, Generator, VisitGenerator, VisitMutGenerator,
};
use schema::{Module, REnum, RStruct, RType, Schema};
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
    runners: Vec<Box<dyn Runner>>,
}

type GeneratedStream = (/* output path */ PathBuf, TokenStream);
type DataStream = (/* output path */ PathBuf, Vec<u8>);

// TODO: remove me
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum GeneratorOutput {
    None,
    Info(Vec<u8>),
    Data(DataStream),
    Stream(GeneratedStream),
}

// TODO: remove me
#[allow(dead_code)]
impl GeneratorOutput {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn expect_none(&self) {
        assert!(self.is_none());
    }

    pub fn to_info(&self) -> &[u8] {
        if let Self::Info(it) = self {
            it
        } else {
            panic!();
        }
    }

    pub fn to_data(&self) -> &DataStream {
        if let Self::Data(it) = self {
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

    pub fn into_info(self) -> Vec<u8> {
        if let Self::Info(it) = self {
            it
        } else {
            panic!();
        }
    }

    pub fn into_data(self) -> DataStream {
        if let Self::Data(it) = self {
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
    ty_table: TypeTable,
    ident_table: IdentTable,
    schema: RefCell<Schema>,
    mods: RefCell<Vec<Module>>,
}

struct CodegenResult {
    schema: Schema,
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

        Self {
            ty_table,
            ident_table,
            schema: RefCell::new(Schema::default()),
            mods: RefCell::new(mods),
        }
    }

    fn find(&self, key: &TypeName) -> Option<TypeRef> {
        self.type_id(key).map(|id| TypeRef::clone(&self.ty_table[*id]))
    }

    fn type_id<'b>(&'b self, key: &'b TypeName) -> Option<&'b TypeId> {
        self.ident_table.get(key)
    }
}

trait Runner {
    fn name(&self) -> &'static str;
    fn run(&mut self, ctx: &CodegenCtx) -> Result<GeneratorOutput>;
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
    fn pass<P>(mut self, pass: P) -> Self
    where
        P: Pass + Runner + 'static,
    {
        self.runners.push(Box::new(pass));
        self
    }

    #[must_use]
    fn gen<G>(mut self, generator: G) -> Self
    where
        G: Generator + Runner + 'static,
    {
        self.runners.push(Box::new(generator));
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

        let outputs = self
            .runners
            .into_iter()
            .map(|mut runner| runner.run(&ctx).map(|res| (runner.name(), res)))
            .collect::<Result<Vec<_>>>()?;

        Ok(CodegenResult { outputs, schema: ctx.schema.into_inner() })
    }
}

fn files() -> impl std::iter::Iterator<Item = String> {
    fn path(path: &str) -> String {
        format!("{AST_CRATE}/src/ast/{path}.rs")
    }

    vec![path("literal"), path("js"), path("ts"), path("jsx")].into_iter()
}

fn write_generated_streams(
    streams: impl IntoIterator<Item = GeneratedStream>,
) -> std::io::Result<()> {
    for (path, stream) in streams {
        let content = pprint(&stream);
        write_all_to(content.as_bytes(), path.into_os_string().to_str().unwrap())?;
    }
    Ok(())
}

fn write_data_streams(streams: impl IntoIterator<Item = DataStream>) -> std::io::Result<()> {
    for (path, content) in streams {
        write_all_to(&content, path.into_os_string().to_str().unwrap())?;
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
        .pass(Linker)
        .pass(BuildSchema)
        .gen(AstKindGenerator)
        .gen(AstBuilderGenerator)
        .gen(ImplGetSpanGenerator)
        .gen(VisitGenerator)
        .gen(VisitMutGenerator)
        .generate()?;

    let (streams, outputs): (Vec<_>, Vec<_>) =
        outputs.into_iter().partition(|it| matches!(it.1, GeneratorOutput::Stream(_)));

    let (binaries, _): (Vec<_>, Vec<_>) =
        outputs.into_iter().partition(|it| matches!(it.1, GeneratorOutput::Data(_)));

    if !cli_options.dry_run {
        write_generated_streams(streams.into_iter().map(|it| it.1.into_stream()))?;
        write_data_streams(binaries.into_iter().map(|it| it.1.into_data()))?;
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

fn output(krate: &str, path: &str) -> PathBuf {
    std::path::PathBuf::from_iter(vec![krate, "src", "generated", path])
}
