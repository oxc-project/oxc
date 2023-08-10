use std::{collections::BTreeMap, env, path::Path, rc::Rc, sync::Arc};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_query::{schema, Adapter};
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use trustfall::{execute_query, FieldValue, TryIntoStruct};

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_query --example simple`
// or `cargo watch -x "run -p oxc_query --example simple"`

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("{name} not found"));
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    let program = allocator.alloc(ret.program);
    let semantic_ret =
        SemanticBuilder::new(&source_text, source_type).with_trivias(ret.trivias).build(program);

    let adapter = Adapter::new(Rc::new(semantic_ret.semantic), vec![Some("index".to_string())]);

    let args: BTreeMap<Arc<str>, FieldValue> = BTreeMap::new();

    let adapter = Arc::from(&adapter);

    #[allow(clippy::items_after_statements)]
    #[derive(Debug, PartialOrd, Ord, PartialEq, Eq, serde::Deserialize)]
    struct Output {
        assignment_to_variable_name: String,
    }

    let mut results: Vec<Output> = execute_query(
        schema(),
        adapter,
        r#"
query {
    File {
        variable_declaration {
            left {
                assignment_to_variable_name @filter(op: "is_not_null") @output
            }
        }
    }
}
        "#,
        args,
    )
    .expect("to successfully execute the query")
    .map(|row| row.try_into_struct().expect("shape mismatch"))
    .collect::<Vec<_>>();

    results.sort_unstable();

    println!("{results:#?}");
}
