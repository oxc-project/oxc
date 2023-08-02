use std::{path::PathBuf, sync::Arc};

use itertools::Itertools;
use oxc_allocator::Allocator;
use oxc_diagnostics::{
    miette::{miette, Diagnostic, ErrorHook, NamedSource},
    Error, GraphicalReportHandler,
};
use oxc_semantic::{Reference, Semantic, SemanticBuilder, SymbolFlags, SymbolId};
use oxc_span::{Atom, SourceType};

pub struct SemanticTester {
    allocator: Allocator,
    source_type: SourceType,
    // source_text: String,
    source_text: &'static str,
    module_builder: bool,
}

impl SemanticTester {
    pub fn ts(source_text: &'static str) -> Self {
        Self::new(source_text, SourceType::default().with_module(true).with_typescript(true))
    }
    pub fn js(source_text: &'static str) -> Self {
        Self::new(source_text, SourceType::default().with_module(true))
    }

    pub fn new(source_text: &'static str, source_type: SourceType) -> Self {
        Self { allocator: Allocator::default(), source_type, source_text, module_builder: true }
    }

    /// Set the [`SourceType`] to TypeScript (or JavaScript, using `false`)
    pub fn with_typescript(mut self, yes: bool) -> Self {
        self.source_type = SourceType::default().with_typescript(yes);
        self
    }

    /// Mark the [`SourceType`] as JSX
    pub fn with_jsx(mut self, yes: bool) -> Self {
        self.source_type = self.source_type.with_jsx(yes);
        self
    }

    /// Set [`SemanticBuilder`]'s `with_module_record_builder` option
    pub fn with_module_record_builder(mut self, yes: bool) -> Self {
        self.module_builder = yes;
        self
    }

    /// Parse the source text and produce a new [`Semantic`]
    pub fn build(&self) -> Semantic<'_> {
        let parse =
            oxc_parser::Parser::new(&self.allocator, &self.source_text, self.source_type).parse();

        if !parse.errors.is_empty() {
            panic!(
                "\n Failed to parse source:\n{}\n\n{}",
                self.source_text,
                parse
                    .errors
                    .iter()
                    .map(|e| format!("{e}"))
                    .intersperse("\n\n".to_owned())
                    .collect::<String>()
            );
        }
        let program = self.allocator.alloc(parse.program);
        let semantic_ret = SemanticBuilder::new(&self.source_text, self.source_type)
            .with_check_syntax_error(true)
            .with_trivias(&parse.trivias)
            .with_module_record_builder(self.module_builder)
            .build(program);

        if !semantic_ret.errors.is_empty() {
            let report = self.wrap_diagnostics(semantic_ret.errors);
            panic!(
                "Semantic analysis failed:\n\n{}",
                report
                    .iter()
                    .map(|r| r.to_string())
                    .intersperse("\n\n".to_owned())
                    .collect::<String>()
            );
        };

        semantic_ret.semantic
    }

    pub fn has_root_symbol(&self, name: &str) -> SymbolTester {
        SymbolTester::new_at_root(self.build(), name, self.source_text)
    }
    /// Finds some symbol by name in the source code.
    /// 
    /// ## Fails
    /// 1. No symbol with the given name exists,
    /// 2. More than one symbol with the given name exists, so a symbol cannot
    ///    be uniquely obtained.
    pub fn has_some_symbol(&self, name: &str) -> SymbolTester {
        SymbolTester::new_unique(self.build(), name, self.source_text)
    }

    fn wrap_diagnostics(&self, diagnostics: Vec<Error>) -> Vec<Error> {
        let name = "test".to_owned()
            + match (self.source_type.is_javascript(), self.source_type.is_jsx()) {
                (true, true) => ".jsx",
                (true, false) => ".js",
                (false, true) => ".tsx",
                (false, false) => ".ts",
            };

        let source = Arc::new(NamedSource::new(name, self.source_text.to_owned()));
        let diagnostics = diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.with_source_code(Arc::clone(&source)))
            .collect();
        diagnostics
    }
}

pub struct SymbolTester<'a> {
    semantic: Semantic<'a>,
    target: String,
    source: &'static str,
    data: Result<SymbolId, oxc_diagnostics::Error>,
}
// enum ReferenceTesterData {
//     Found(SymbolId),
//     Err(oxc_diagnostics::Error)
// }
impl<'a> SymbolTester<'a> {
    pub fn new_at_root(semantic: Semantic<'a>, target: &str, source: &'static str) -> Self {
        let decl =
            semantic.scopes().get_binding(semantic.scopes().root_scope_id(), &Atom::from(target));
        let data = decl.map_or_else(
            || Err(miette!("Could not find declaration for {target}")),
            |decl| Ok(decl),
        );

        SymbolTester { semantic, target: target.to_string(), data, source }
    }

    pub fn new_unique(semantic: Semantic<'a>, target: &str, source: &'static str) -> Self {
        let symbols_with_target_name: Vec<_> =
            semantic.scopes().iter_bindings().filter(|(_, _, name)| name == &target).collect();
        let data = match symbols_with_target_name.len() {
            0 => Err(miette!("Could not find declaration for {target}")),
            1 => Ok(symbols_with_target_name.iter().map(|(_, symbol_id, _)| *symbol_id).nth(0).unwrap()),
            n if n > 1 => Err(miette!("Couldn't uniquely resolve symbol id for target {target}; {n} symbols with that name are declared in the source.")),
            _ => unreachable!()
        };

        SymbolTester { semantic, target: target.to_string(), data, source }
    }

    /// Checks if the resolved symbol contains all flags in `flags`, using [`SymbolFlags::contains()`]
    pub fn contains_flags(mut self, flags: SymbolFlags) -> Self {
        self.data = match self.data {
            Ok(symbol_id) => {
                let found_flags = self.semantic.symbols().get_flag(symbol_id);
                if found_flags.contains(flags) {
                    Ok(symbol_id)
                } else {
                    Err(miette!(
                        "Expected {} to contain flags {:?}, but it had {:?}",
                        self.target,
                        flags,
                        found_flags
                    ))
                }
            }
            err => err,
        };
        self
    }

    pub fn intersects_flags(mut self, flags: SymbolFlags) -> Self {
        self.data = match self.data {
            Ok(symbol_id) => {
                let found_flags = self.semantic.symbols().get_flag(symbol_id);
                if found_flags.intersects(flags) {
                    Ok(symbol_id)
                } else {
                    Err(miette!(
                        "Expected {} to intersect with flags {:?}, but it had {:?}",
                        self.target,
                        flags,
                        found_flags
                    ))
                }
            }
            err => err,
        };
        self
    }

    pub fn has_number_of_reads(self, ref_count: usize) -> Self {
        self.has_number_of_references_where(ref_count, |r| r.is_read())
    }

    pub fn has_number_of_write(self, ref_count: usize) -> Self {
        self.has_number_of_references_where(ref_count, |r| r.is_write())
    }

    pub fn has_number_of_references(self, ref_count: usize) -> Self {
        self.has_number_of_references_where(ref_count, |_| true)
    }

    pub fn has_number_of_references_where<F>(mut self, ref_count: usize, filter: F) -> Self
    where
        F: FnMut(&Reference) -> bool,
    {
        self.data = match self.data {
            Ok(symbol_id) => {
                let refs = {
                    self.semantic
                        .symbols()
                        .get_resolved_reference_ids(symbol_id)
                        .iter()
                        .map(|r_id| self.semantic.symbols().get_reference(*r_id).clone())
                };
                let num_accepted = refs.filter(filter).count();
                if num_accepted == ref_count {
                    Ok(symbol_id)
                } else {
                    Err(miette!("Expected to find {ref_count} acceptable references, but only found {num_accepted}"))
                }
            }
            e => e,
        };
        self
    }

    /// Complete the test case. Will panic if any of the previously applied
    /// assertions failed.
    pub fn test(self) {
        // self.into::<Result<_, Error>>().unwrap()
        let res: Result<_, _> = self.into();

        // res.map_err(|e| {
        //     let s = String::new();
        //     // let res = GraphicalReportHandler::new().render_report(s, &e).unwrap();
        // });
        res.unwrap();
    }
}

impl<'a> Into<Result<(), Error>> for SymbolTester<'a> {
    fn into(self) -> Result<(), Error> {
        self.data.map(|_| {}).map_err(|e| e.with_source_code(self.source))
    }
}

// fn graphic_hook(diag: dyn Diagnostic) -> GraphicalReportHandler {
//     Box::new(GraphicalReportHandler::new().tab_width(2))
// }
