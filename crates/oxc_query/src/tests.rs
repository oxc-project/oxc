use std::{collections::BTreeMap, rc::Rc, sync::Arc};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use trustfall::{execute_query, provider::check_adapter_invariants, FieldValue, TryIntoStruct};

use crate::{adapter::schema, Adapter};

fn run_query<T: for<'de> serde::Deserialize<'de> + std::cmp::Ord>(
    code: &str,
    query: &str,
) -> Vec<T> {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_module(true).with_jsx(true).with_typescript(true);
    let ret = Parser::new(&allocator, code, source_type).parse();
    let program = allocator.alloc(ret.program);
    let semantic_ret =
        SemanticBuilder::new(code, source_type).with_trivias(ret.trivias).build(program);

    let adapter = Adapter {
        path_components: vec![Some("index".to_string())],
        semantic: Rc::new(semantic_ret.semantic),
    };

    let args: BTreeMap<Arc<str>, FieldValue> = BTreeMap::new();

    let adapter = Arc::from(&adapter);

    let mut results: Vec<T> = execute_query(schema(), adapter, query, args)
        .expect("to successfully execute the query")
        .map(|row| row.try_into_struct::<T>().expect("shape mismatch"))
        .collect::<Vec<_>>();

    results.sort_unstable();

    results
}

#[test]
fn test_path_query() {
    #[derive(Debug, PartialOrd, Ord, PartialEq, Eq, serde::Deserialize)]
    struct Output {
        name: String,
        is_first: bool,
        is_last: bool,
    }
    let results = run_query::<Output>(
        "const apple = 1;",
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
    );

    assert_eq!(vec![Output { name: "index".into(), is_first: true, is_last: true },], results);
}

#[test]
fn test_variable_query() {
    #[derive(Debug, PartialOrd, Ord, PartialEq, Eq, serde::Deserialize)]
    struct Output {
        assignment_to_variable_name: String,
        span_asmt_type_start: u64,
        span_asmt_type_end: u64,
        span_start: u64,
        span_end: u64,
        __typename: String,
    }
    let results = run_query::<Output>(
        "const apple = 1;",
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

                    __typename @output

                    span_: span {
                        start @output
                        end @output
                    }
                }
            }
        }
        "#,
    );

    assert_eq!(
        vec![Output {
            assignment_to_variable_name: "apple".to_owned(),
            span_asmt_type_start: 6,
            span_asmt_type_end: 11,
            span_start: 6,
            span_end: 15,
            __typename: "VariableDeclarationAST".to_owned()
        }],
        results
    );
}

#[test]
fn test_object_literal_ast() {
    #[derive(Debug, PartialOrd, Ord, PartialEq, Eq, serde::Deserialize)]
    struct Output {
        __typename: String,
        value_typename: String,
    }
    let results = run_query::<Output>(
        "const colors = { blue: 1, green: 2, red: 3 };",
        r#"
        {
            File {
                ast_node {
                    ... on ObjectLiteralAST {
                        value(key: "blue") {
                            value_typename: __typename @output
                        }
                        __typename @output
                    }
                }
            }
        }
        "#,
    );

    assert_eq!(
        vec![Output {
            __typename: "ObjectLiteralAST".to_owned(),
            value_typename: "NumberLiteral".to_owned()
        }],
        results
    );
}

#[test]
fn test_parent_query() {
    #[derive(Debug, PartialOrd, Ord, PartialEq, Eq, serde::Deserialize)]
    struct Output {
        type_: String,
        tn1: String,
        tn2: String,
    }
    let results = run_query::<Output>(
        "interface MyGreatInterface { myGreatProperty: number }",
        r#"
        query {
            File {
                ast_node {
                    ... on TypeAnnotationAST {
                        type {
                            span {
                                type_: str @output
                            }
                        }
                        parent {
                            tn1: __typename @output
                            parent {
                                tn2: __typename @output
                            }
                        }
                    }
                }
            }
        }
        "#,
    );

    assert_eq!(
        vec![Output {
            type_: "number".to_owned(),
            tn1: "ASTNode".to_owned(),
            tn2: "InterfaceAST".to_owned()
        }],
        results
    );
}

#[test]
fn test_invariants() {
    let file_path = "/apple/orange.tsx";
    let source_text = "const apple = 1;";

    let allocator = Allocator::default();
    let source_type = SourceType::from_path(file_path).unwrap();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let program = allocator.alloc(ret.program);
    let semantic_ret =
        SemanticBuilder::new(source_text, source_type).with_trivias(ret.trivias).build(program);

    let adapter = Adapter { path_components: vec![], semantic: Rc::new(semantic_ret.semantic) };
    check_adapter_invariants(schema(), &adapter);
}
