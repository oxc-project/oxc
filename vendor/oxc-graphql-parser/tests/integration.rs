use std::fs;
use std::path::Path;
use std::path::PathBuf;

use oxc_graphql_parser::Allocator;
use oxc_graphql_parser::Error;
use oxc_graphql_parser::Parser;
use oxc_graphql_parser::ast;

#[test]
fn parser_parses_object_type_definition() {
    let source = r#"
type Query {
  hello(name: String = "world"): String
}
"#;
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source).parse();

    assert_eq!(ast.errors().len(), 0);
    let document = ast.document();
    assert_eq!(document.definitions.len(), 1);

    let ast::Definition::ObjectType(object) = &document.definitions[0] else {
        panic!("expected object type definition");
    };
    assert_eq!(object.name.as_str(), "Query");
    assert_eq!(object.fields.len(), 1);
    assert_eq!(object.fields[0].name.as_str(), "hello");
    assert_eq!(object.fields[0].arguments[0].name.as_str(), "name");
}

#[test]
fn parser_parses_query_variables_and_used_variables() {
    let source = r#"
query GraphQuery($graph_id: ID!, $variant: String) {
  service(id: $graph_id) {
    schema(tag: $variant) {
      document
    }
  }
}
"#;
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source).parse();
    assert_eq!(ast.errors().len(), 0);

    let ast::Definition::Operation(operation) = &ast.document().definitions[0] else {
        panic!("expected operation definition");
    };
    assert_eq!(operation.name.as_ref().unwrap().as_str(), "GraphQuery");
    assert_eq!(operation.variable_definitions.len(), 2);

    let mut used = Vec::new();
    collect_variables(operation.selection_set.as_ref().unwrap(), &mut used);
    assert_eq!(used, ["graph_id", "variant"]);
}

#[test]
fn parser_parses_selection_set_and_type_roots() {
    let allocator = Allocator::default();
    let selection = Parser::new(&allocator, "{ product { name } }").parse_selection_set();
    assert_eq!(selection.errors().len(), 0);
    assert_eq!(selection.field_set().selections.len(), 1);

    let ty = Parser::new(&allocator, "[String!]!").parse_type();
    assert_eq!(ty.errors().len(), 0);
    assert!(matches!(ty.ty(), ast::Type::NonNull(_)));
}

#[test]
fn parser_parses_experimental_fragment_arguments() {
    let source = r#"
fragment variableProfilePic($size: Int) on User {
  profilePic(size: $size)
}

query Q {
  user {
    ...variableProfilePic(size: 100)
  }
}
"#;
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source).experimental_fragment_arguments(true).parse();
    assert_eq!(ast.errors().len(), 0);

    let ast::Definition::Fragment(fragment) = &ast.document().definitions[0] else {
        panic!("expected fragment definition");
    };
    assert_eq!(fragment.name.as_str(), "variableProfilePic");
    assert_eq!(fragment.variable_definitions.len(), 1);
    assert_eq!(fragment.variable_definitions[0].variable.name.as_str(), "size");

    let ast::Definition::Operation(operation) = &ast.document().definitions[1] else {
        panic!("expected operation definition");
    };
    let ast::Selection::Field(user) = &operation.selection_set.as_ref().unwrap().selections[0]
    else {
        panic!("expected field");
    };
    let ast::Selection::FragmentSpread(spread) =
        &user.selection_set.as_ref().unwrap().selections[0]
    else {
        panic!("expected fragment spread");
    };
    assert_eq!(spread.name.as_str(), "variableProfilePic");
    assert_eq!(spread.arguments.len(), 1);
    assert_eq!(spread.arguments[0].name.as_str(), "size");
}

#[test]
fn parser_object_value_respects_recursion_limit() {
    // Deeply-nested object values must trip the recursion limit and report an
    // error instead of overflowing the stack — matching the guard already
    // applied to list values and selection sets.
    let depth = 100;
    let mut source = String::from("query { field(arg: ");
    source.push_str(&"{ nested: ".repeat(depth));
    source.push('1');
    source.push_str(&" }".repeat(depth));
    source.push_str(") }");

    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, &source).recursion_limit(16).parse();

    assert!(
        ast.errors().any(Error::is_limit),
        "expected a recursion limit error for deeply nested object values"
    );
    assert!(ast.recursion_limit().high > ast.recursion_limit().limit);

    // Nesting within the limit must still parse without a recursion error.
    let ok = "query { field(arg: { a: { b: { c: 1 } } }) }";
    let ok_ast = Parser::new(&allocator, ok).recursion_limit(16).parse();
    assert!(!ok_ast.errors().any(Error::is_limit));
}

#[test]
fn parser_rejects_fragment_arguments_without_flag() {
    let source = "query Q { user { ...spread(size: 100) } }";
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source).parse();
    assert!(ast.errors().len() > 0);
}

#[test]
fn parser_collects_comment_spans_in_document_order() {
    let source = r#"# leading
query Q {
  # inside selection set
  field # trailing
  # before closing brace
}
# between definitions
type T {
  name: String
}
# at end of document"#;
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source).parse();
    assert_eq!(ast.errors().len(), 0);

    let comments = ast
        .comments()
        .iter()
        .map(|span| &source[span.start as usize..span.end as usize])
        .collect::<Vec<_>>();
    assert_eq!(
        comments,
        [
            "# leading",
            "# inside selection set",
            "# trailing",
            "# before closing brace",
            "# between definitions",
            "# at end of document",
        ]
    );
}

#[test]
fn parser_string_values() {
    // Escaped strings and block strings with `\r` line endings are unescaped
    // or normalized into arena-allocated values; strings that need no
    // rewriting are borrowed directly from the source text.
    let source = "\"plain\"\nscalar A\n\"esc\\u0041ped \\n\\\"quote\\\" end\"\nscalar B\n\"\"\"one\r\ntwo\rthree\nfour\"\"\"\nscalar C\n\"\"\"block unchanged\"\"\"\nscalar D\n\"trailing \\\"\"\nscalar E\n\"\\\"\"\nscalar F";
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source).parse();
    assert_eq!(ast.errors().len(), 0);

    let descriptions = ast
        .document()
        .definitions
        .iter()
        .map(|definition| {
            let ast::Definition::ScalarType(scalar) = definition else {
                panic!("expected scalar definition");
            };
            scalar.description.as_ref().unwrap().value
        })
        .collect::<Vec<_>>();
    assert_eq!(
        descriptions,
        [
            "plain",
            "escAped \n\"quote\" end",
            "one\ntwo\nthree\nfour",
            "block unchanged",
            "trailing \"",
            "\"",
        ]
    );
}

#[test]
fn parser_collects_comments_without_duplicates_on_lookahead() {
    // `extend` parsing peeks ahead multiple times; comments in between must be
    // recorded only once.
    let source = "# before extend\nextend type T { name: String }";
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source).parse();
    assert_eq!(ast.errors().len(), 0);
    let end = u32::try_from("# before extend".len()).unwrap();
    assert_eq!(ast.comments(), [ast::Span::new(0, end)]);
}

#[test]
fn parser_collects_no_comments_when_absent() {
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, "type T { name: String }").parse();
    assert_eq!(ast.errors().len(), 0);
    assert!(ast.comments().is_empty());
}

#[test]
fn parser_comment_spans_end_before_line_terminators() {
    for line_terminator in ["\n", "\r\n", "\r"] {
        let source = format!("# comment{line_terminator}type T {{ name: String }}");
        let allocator = Allocator::default();
        let ast = Parser::new(&allocator, &source).parse();
        assert_eq!(ast.errors().len(), 0);
        let end = u32::try_from("# comment".len()).unwrap();
        assert_eq!(ast.comments(), [ast::Span::new(0, end)]);
    }
}

#[test]
fn parser_collects_comments_for_selection_set_and_type_roots() {
    let allocator = Allocator::default();
    let selection = Parser::new(&allocator, "{ field # inside\n}").parse_selection_set();
    assert_eq!(selection.errors().len(), 0);
    assert_eq!(selection.comments().len(), 1);

    let ty = Parser::new(&allocator, "String").parse_type();
    assert_eq!(ty.errors().len(), 0);
    assert!(ty.comments().is_empty());
}

#[test]
fn definition_and_selection_spans_match_inner_nodes() {
    let source = r#"query Q {
  field
  ...spread
  ... on T {
    inline
  }
}
type T {
  name: String
}
extend type T {
  extra: String
}"#;
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source).parse();
    assert_eq!(ast.errors().len(), 0);

    let definitions = &ast.document().definitions;
    let ast::Definition::Operation(operation) = &definitions[0] else {
        panic!("expected operation definition");
    };
    assert_eq!(definitions[0].span(), operation.span);
    let ast::Definition::ObjectType(object) = &definitions[1] else {
        panic!("expected object type definition");
    };
    assert_eq!(definitions[1].span(), object.span);
    let ast::Definition::ObjectTypeExtension(extension) = &definitions[2] else {
        panic!("expected object type extension");
    };
    assert_eq!(definitions[2].span(), extension.span);

    let selections = &operation.selection_set.as_ref().unwrap().selections;
    let ast::Selection::Field(field) = &selections[0] else {
        panic!("expected field");
    };
    assert_eq!(selections[0].span(), field.span);
    let ast::Selection::FragmentSpread(spread) = &selections[1] else {
        panic!("expected fragment spread");
    };
    assert_eq!(selections[1].span(), spread.span);
    let ast::Selection::InlineFragment(inline) = &selections[2] else {
        panic!("expected inline fragment");
    };
    assert_eq!(selections[2].span(), inline.span);
}

#[test]
fn parser_parses_directives_on_directive_definitions() {
    let source = "directive @foo @bar @baz on FIELD";
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source).parse();
    assert_eq!(ast.errors().len(), 0);

    let ast::Definition::Directive(directive) = &ast.document().definitions[0] else {
        panic!("expected directive definition");
    };
    assert_eq!(directive.name.as_str(), "foo");
    assert_eq!(directive.directives.len(), 2);
    assert_eq!(directive.directives[0].name.as_str(), "bar");
    assert_eq!(directive.directives[1].name.as_str(), "baz");
}

#[test]
fn parser_parses_directive_extension() {
    let source = "extend directive @foo @bar @baz";
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source).parse();
    assert_eq!(ast.errors().len(), 0);

    let ast::Definition::DirectiveExtension(extension) = &ast.document().definitions[0] else {
        panic!("expected directive extension");
    };
    assert_eq!(extension.name.as_str(), "foo");
    assert_eq!(extension.directives.len(), 2);
    assert_eq!(extension.directives[0].name.as_str(), "bar");
    assert_eq!(extension.directives[1].name.as_str(), "baz");
}

#[test]
fn parser_rejects_empty_directive_extension() {
    let source = "extend directive @foo";
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source).parse();
    assert!(ast.errors().len() > 0);
}

#[test]
fn parser_parses_variable_definition_descriptions() {
    let source = r#"query Q("""the id""" $id: ID!) { node(id: $id) { name } }"#;
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source).parse();
    assert_eq!(ast.errors().len(), 0);

    let ast::Definition::Operation(operation) = &ast.document().definitions[0] else {
        panic!("expected operation definition");
    };
    assert_eq!(operation.variable_definitions.len(), 1);
    let description = operation.variable_definitions[0].description.as_ref().unwrap();
    assert_eq!(description.value, "the id");
}

#[test]
fn parser_rejects_description_on_shorthand_query() {
    let source = r#""""doc""" { f }"#;
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source).parse();
    assert!(ast.errors().len() > 0);
}

#[test]
fn parser_rejects_description_on_extension() {
    let source = r#""""doc""" extend type Foo @bar"#;
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, source).parse();
    assert!(ast.errors().len() > 0);
}

#[test]
fn parser_ok_fixtures_have_no_errors() {
    for path in graphql_files("parser/ok") {
        let source = fs::read_to_string(&path).unwrap();
        let allocator = Allocator::default();
        let ast = Parser::new(&allocator, &source).parse();
        let errors = ast.errors().collect::<Vec<_>>();
        assert!(errors.is_empty(), "{}: {errors:?}", path.display());
    }
}

#[test]
fn parser_err_fixtures_have_errors() {
    for path in graphql_files("parser/err") {
        let source = fs::read_to_string(&path).unwrap();
        let allocator = Allocator::default();
        let ast = Parser::new(&allocator, &source).parse();
        assert!(ast.errors().len() > 0, "{}", path.display());
    }
}

#[test]
fn selection_set_recovers_from_invalid_selection_start() {
    // Tokens that cannot start a selection used to make the selection-set
    // loop spin without consuming anything: an infinite loop in release
    // builds and a debug assert in debug builds.
    for source in [r#"{ name: "value" }"#, "{ ! }", "{ 1 }", r#"{ a "b" c }"#] {
        let allocator = Allocator::default();
        let ast = Parser::new(&allocator, source).parse();
        assert!(ast.errors().len() > 0, "{source}");
    }

    // Recovery keeps the surrounding selections.
    let allocator = Allocator::default();
    let ast = Parser::new(&allocator, r#"{ a "b" c }"#).parse();
    let ast::Definition::Operation(operation) = &ast.document().definitions[0] else {
        panic!("expected operation definition");
    };
    let selections = &operation.selection_set.as_ref().unwrap().selections;
    assert_eq!(selections.len(), 2);
}

#[test]
fn all_fixture_files_parse_without_panicking() {
    // Feed every fixture file through the parser, including the `.txt`
    // expectation files, which act as garbage input.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_data");
    for dir in ["lexer/ok", "lexer/err", "parser/ok", "parser/err"] {
        for entry in fs::read_dir(root.join(dir)).unwrap() {
            let path = entry.unwrap().path();
            let Ok(source) = fs::read_to_string(&path) else { continue };
            let allocator = Allocator::default();
            let _ast = Parser::new(&allocator, &source).parse();
        }
    }
}

#[test]
#[ignore]
fn ecosystem_graphql_corpus_has_no_parse_errors() {
    let root = std::env::var_os("OXC_GRAPHQL_ECOSYSTEM_REPOS")
        .map(PathBuf::from)
        .expect("set OXC_GRAPHQL_ECOSYSTEM_REPOS to an ecosystem-ci repos directory");
    let mut files = Vec::new();
    collect_graphql_files(&root, &mut files);

    let mut failures = Vec::new();
    for path in &files {
        let source = fs::read_to_string(path).unwrap();
        let allocator = Allocator::default();
        let ast = Parser::new(&allocator, &source).parse();
        let errors = ast.errors().collect::<Vec<_>>();
        if !errors.is_empty() {
            failures.push(format!("{}: {errors:?}", path.display()));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} ecosystem GraphQL files failed to parse:\n{}",
        failures.len(),
        files.len(),
        failures.join("\n")
    );
}

fn collect_variables<'a>(selection_set: &'a ast::SelectionSet<'_>, output: &mut Vec<&'a str>) {
    for selection in &selection_set.selections {
        if let ast::Selection::Field(field) = selection {
            for argument in &field.arguments {
                collect_variable_value(argument.value.as_ref(), output);
            }
            if let Some(selection_set) = &field.selection_set {
                collect_variables(selection_set, output);
            }
        }
    }
}

fn collect_variable_value<'a>(value: Option<&'a ast::Value<'_>>, output: &mut Vec<&'a str>) {
    match value {
        Some(ast::Value::Variable(variable)) => output.push(variable.name.as_str()),
        Some(ast::Value::List(list)) => {
            for value in &list.values {
                collect_variable_value(Some(value), output);
            }
        }
        Some(ast::Value::Object(object)) => {
            for field in &object.fields {
                collect_variable_value(field.value.as_ref(), output);
            }
        }
        _ => {}
    }
}

fn graphql_files(path: &str) -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_data").join(path);
    let mut files = fs::read_dir(dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "graphql"))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn collect_graphql_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if matches!(name, ".git" | "node_modules" | "target") {
                continue;
            }
            collect_graphql_files(&path, files);
        } else if path
            .extension()
            .is_some_and(|extension| matches!(extension.to_str(), Some("gql" | "graphql")))
        {
            files.push(path);
        }
    }
    files.sort();
}
