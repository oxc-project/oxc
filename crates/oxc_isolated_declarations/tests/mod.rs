mod deno;

use std::{fmt::Write, path::Path, sync::Arc};

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions};
use oxc_parser::Parser;
use oxc_span::{SourceCode, SourceCodeBuf, SourceType};

fn transform(path: &Path, source: SourceCode<'_>) -> String {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let parser_ret = Parser::from_source(&allocator, source, source_type).parse();
    assert!(
        parser_ret.diagnostics.is_empty(),
        "Parser errors for {}: {:?}",
        path.display(),
        parser_ret.diagnostics
    );

    let id_ret =
        IsolatedDeclarations::new(&allocator, IsolatedDeclarationsOptions { strip_internal: true })
            .build(&parser_ret.program);
    let code = Codegen::new().build(&id_ret.program).code;

    let mut snapshot =
        format!("```\n==================== .D.TS ====================\n\n{code}\n\n");
    if !id_ret.diagnostics.is_empty() {
        let source = Arc::new(source.as_str().to_string());
        let error_messages = id_ret
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.clone().render_with_source_code(Arc::clone(&source)))
            .fold(String::new(), |output, error| output + &error);

        write!(
            snapshot,
            "==================== Errors ====================\n{error_messages}\n\n```"
        )
        .unwrap();
    }

    snapshot
}

#[test]
fn snapshots() {
    insta::glob!("fixtures/*.{ts,tsx}", |path| {
        let source = SourceCodeBuf::read(path).unwrap();
        let snapshot = transform(path, source.as_source_code());
        let name = path.file_stem().unwrap().to_str().unwrap();
        insta::with_settings!({ prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
            insta::assert_snapshot!(name, snapshot);
        });
    });
}
