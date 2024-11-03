use std::{path::Path, str::FromStr};

use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{ESTarget, TransformOptions, Transformer};

use crate::run;

pub(crate) fn test(source_text: &str, target: &str) -> String {
    let source_type = SourceType::default();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let mut program = ret.program;
    let (symbols, scopes) =
        SemanticBuilder::new().build(&program).semantic.into_symbol_table_and_scope_tree();
    let options = TransformOptions::from(ESTarget::from_str(target).unwrap());
    Transformer::new(&allocator, Path::new(""), options).build_with_symbols_and_scopes(
        symbols,
        scopes,
        &mut program,
    );
    CodeGenerator::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&program)
        .code
}

#[test]
fn es2015() {
    use std::fmt::Write;

    let cases = [
        ("es5", "() => {}"),
        ("es2015", "a ** b"),
        ("es2016", "async function foo() {}"),
        ("es2017", "({ ...x })"),
        ("es2018", "try {} catch {}"),
        ("es2019", "a ?? b"),
        ("es2020", "a ||= b"),
        ("es2021", "class foo { static {} }"),
    ];

    // Test no transformation for esnext.
    for (_, case) in cases {
        assert_eq!(run(case, SourceType::mjs()), test(case, "esnext"));
    }

    let snapshot = cases.iter().enumerate().fold(String::new(), |mut w, (i, (target, case))| {
        let result = test(case, target);
        write!(w, "########## {i} {target}\n{case}\n----------\n{result}\n").unwrap();
        w
    });

    insta::with_settings!({ prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
        insta::assert_snapshot!("es_target", snapshot);
    });
}
