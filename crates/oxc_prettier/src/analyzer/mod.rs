use oxc_ast::ast::Program;
pub use table_imports::TableImports;

pub mod table_imports;

/// Analyzer stores context values for entire program
/// to make formatting better
#[derive(Debug, Default)]
pub struct Analysis {
	/// [TableImports]
	pub ti: TableImports,
}

impl Analysis {
	pub fn for_program(program: &Program) -> Self {
		Self {
			ti: TableImports::analyze(program)
		}
	}
}
