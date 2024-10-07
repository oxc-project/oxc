mod deno;

use std::{fs, path::Path, sync::Arc};

use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CommentOptions};
use oxc_isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

fn transform(path: &Path, source_text: &str) -> String {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let parser_ret = Parser::new(&allocator, source_text, source_type).parse();

    let id_ret = IsolatedDeclarations::new(
        &allocator,
        source_text,
        &parser_ret.trivias,
        IsolatedDeclarationsOptions { strip_internal: true },
    )
    .build(&parser_ret.program);
    let code = CodeGenerator::new()
        .enable_comment(
            source_text,
            parser_ret.trivias,
            CommentOptions { preserve_annotate_comments: false },
        )
        .build(&id_ret.program)
        .code;

    let mut snapshot =
        format!("```\n==================== .D.TS ====================\n\n{code}\n\n");
    if !id_ret.errors.is_empty() {
        let source = Arc::new(source_text.to_string());
        let error_messages = id_ret
            .errors
            .iter()
            .map(|d| d.clone().with_source_code(Arc::clone(&source)))
            .map(|error| format!("{error:?}"))
            .collect::<Vec<_>>()
            .join("\n");

        snapshot.push_str(&format!(
            "==================== Errors ====================\n\n{error_messages}\n\n```"
        ));
    }

    snapshot
}

#[test]
fn snapshots() {
    insta::glob!("fixtures/*.{ts,tsx}", |path| {
        let source_text = fs::read_to_string(path).unwrap();
        let snapshot = transform(path, &source_text);
        let name = path.file_stem().unwrap().to_str().unwrap();
        insta::with_settings!({ prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
            insta::assert_snapshot!(name, snapshot);
        });
    });
}
