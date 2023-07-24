#![feature(let_chains)]
#![allow(clippy::redundant_pub_crate)]
mod adapter;
mod edges;
mod entrypoints;
mod properties;
mod util;
mod vertex;

pub use adapter::{schema, Adapter};
pub use vertex::Vertex;

// TODO: Uncomment on next release of trustfall
#[cfg(test)]
mod test {
    use std::{collections::BTreeMap, rc::Rc, sync::Arc};

    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::SourceType;
    use trustfall::{execute_query, FieldValue, TryIntoStruct};

    use crate::{adapter::schema, Adapter};

    #[test]
    fn test_path_query() {
        let code = "const apple = 1;";

        let allocator = Allocator::default();
        let source_type =
            SourceType::default().with_module(true).with_jsx(true).with_typescript(true);
        let ret = Parser::new(&allocator, code, source_type).parse();
        let program = allocator.alloc(ret.program);
        let semantic_ret =
            SemanticBuilder::new(code, source_type).with_trivias(&ret.trivias).build(program);

        let adapter = Adapter {
            path_components: vec![Some("index".to_string())],
            semantic: Rc::new(semantic_ret.semantic),
        };

        let args: BTreeMap<Arc<str>, FieldValue> = BTreeMap::new();

        let adapter = Arc::from(&adapter);

        #[allow(clippy::items_after_statements)]
        #[derive(Debug, PartialOrd, Ord, PartialEq, Eq, serde::Deserialize)]
        struct Output {
            name: String,
            is_first: bool,
            is_last: bool,
        }

        let mut results: Vec<Output> = execute_query(
            schema(),
            adapter,
            r#"
query {
    File {
        last_path_part {
            name @output
            is_first @output
            is_last @output
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

        assert_eq!(vec![Output { name: "index".into(), is_first: true, is_last: true },], results);
    }

    #[test]
    fn test_variable_query() {
        let code = "const apple = 1;";

        let allocator = Allocator::default();
        let source_type =
            SourceType::default().with_module(true).with_jsx(true).with_typescript(true);
        let ret = Parser::new(&allocator, code, source_type).parse();
        let program = allocator.alloc(ret.program);
        let semantic_ret =
            SemanticBuilder::new(code, source_type).with_trivias(&ret.trivias).build(program);

        let adapter = Adapter {
            path_components: vec![Some("index".to_string())],
            semantic: Rc::new(semantic_ret.semantic),
        };

        let args: BTreeMap<Arc<str>, FieldValue> = BTreeMap::new();

        let adapter = Arc::from(&adapter);

        #[allow(clippy::items_after_statements)]
        #[derive(Debug, PartialOrd, Ord, PartialEq, Eq, serde::Deserialize)]
        struct Output {
            assignment_to_variable_name: String,
            span_asmt_type_start: u64,
            span_asmt_type_end: u64,
            entire_span_start: u64,
            entire_span_end: u64,
        }

        let mut results: Vec<Output> = execute_query(
            schema(),
            adapter,
            r#"
query {
    File {
        variable_declaration {
            left {
                assignment_to_variable_name @output
                span_asmt_type_: span {
                    start @output
                    end @output
                }
            }

            entire_span_: entire_span {
                start @output
                end @output
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

        assert_eq!(
            vec![Output {
                assignment_to_variable_name: "apple".to_owned(),
                span_asmt_type_start: 6,
                span_asmt_type_end: 11,
                entire_span_start: 6,
                entire_span_end: 15
            }],
            results
        );
    }

    // #[test]
    // fn test_invariants() {
    // let file_path = "/apple/orange.tsx";
    // let source_text = "const apple = 1;";

    // let allocator = Allocator::default();
    // let source_type = SourceType::from_path(file_path).unwrap();
    // let ret = Parser::new(&allocator, source_text, source_type).parse();
    // let program = allocator.alloc(ret.program);
    // let semantic_ret = SemanticBuilder::new(source_text, source_type)
    //     .with_trivias(&ret.trivias)
    //     .build(program);

    // let adapter = Adapter { path_components: vec![], semantic: Rc::new(semantic_ret.semantic) };
    //     check_adapter_invariants(schema(), &adapter);
    // }
}
