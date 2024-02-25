use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::VisitMut;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use oxc_span::SourceType;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_query --example simple`
// or `cargo watch -x "run -p oxc_query --example simple"`

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    let source_text = "function foo(event) {
		if (this._listeners === undefined) 		return;

		var listeners = this._listeners;
		var listenerArray = listeners[event.type];
		if (listenerArray !== undefined) {
			event.target = this;
			var array = listenerArray.slice(0);
			for (var i$1 = 0, l = array.length; i$1 < l; i++)			{
				array[i$1].call(this, event);
			}
		}
	}";
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    let program = allocator.alloc(ret.program);
    let SemanticBuilderReturn { semantic, .. } =  SemanticBuilder::new(&source_text, source_type).with_trivias(ret.trivias).build(program);

    let mut visualizer = ScopeIdVisualizer { semantic: &semantic };
    visualizer.visit_program(program);

    let codegen_options = CodegenOptions;
    let printed = Codegen::<false>::new(source_text.len(), codegen_options).build(program);
    println!("Printed:\n");
    println!("{printed}");
}

struct ScopeIdVisualizer<'a, 'b> {
    semantic: &'a oxc_semantic::Semantic<'b>,
}

impl<'a> VisitMut<'a> for ScopeIdVisualizer<'_, '_> {
    fn visit_identifier_reference(&mut self, ident: &mut oxc_ast::ast::IdentifierReference) {
        if let Some(ref_id) = ident.reference_id.get() {
            let refer = self.semantic.symbols().get_reference(ref_id);
            if let Some(symbol_id) = refer.symbol_id() {
                let scope_id = self.semantic.symbol_scope(symbol_id);
                ident.name = format!("{}#{}", ident.name, scope_id.raw()).into();
            }
        }
    }

    fn visit_binding_identifier(&mut self, ident: &mut oxc_ast::ast::BindingIdentifier) {
        if let Some(symbol_id) = ident.symbol_id.get() {
            let scope_id = self.semantic.symbol_scope(symbol_id);
            ident.name = format!("{}#{}", ident.name, scope_id.raw()).into();
        }
    }
}