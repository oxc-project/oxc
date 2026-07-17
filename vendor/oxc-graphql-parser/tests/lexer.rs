use std::fs;
use std::path::Path;
use std::path::PathBuf;

use oxc_graphql_parser::Error;
use oxc_graphql_parser::Lexer;
use oxc_graphql_parser::Token;
use oxc_graphql_parser::TokenKind;

#[test]
fn lexer_tests() {
    let source = r#"
type Query {
  hello(name: String = "world"): String
}
"#;
    let (tokens, errors) = Lexer::new(source).lex();
    assert!(errors.is_empty());
    assert!(tokens.iter().any(|token| token.kind() == TokenKind::Name && token.data() == "Query"));
}

#[test]
fn lexer_roundtrip_corpus() {
    // For any input the lexer tokenizes without errors, concatenating all
    // token text must reproduce the source exactly.
    for dir in ["lexer/ok", "lexer/err", "parser/ok", "parser/err"] {
        for path in graphql_files(dir) {
            let source = fs::read_to_string(&path).unwrap();
            let (tokens, errors) = Lexer::new(&source).lex();
            if errors.is_empty() {
                let concatenated: String = tokens.iter().map(Token::data).collect();
                assert_eq!(source, concatenated, "{}", path.display());
            }
        }
    }
}

#[test]
fn unterminated_string() {
    let schema = r#"
type Query {
    name: String
    format: String = "Y-m-d\\TH:i:sP"
}
        "#;
    let (tokens, errors) = Lexer::new(schema).lex();
    assert!(errors.is_empty());
    assert!(tokens.iter().any(|token| {
        token.kind() == TokenKind::StringValue && token.data() == r#""Y-m-d\\TH:i:sP""#
    }));
}

#[test]
fn token_limit() {
    let lexer = Lexer::new("type Query { a a a a a a a a a }").with_limit(10);
    let (tokens, errors) = lexer.lex();
    assert_eq!(tokens.len(), 10);
    assert_eq!(errors, &[Error::limit("token limit reached, aborting lexing", 17)]);
}

#[test]
fn token_limit_exact() {
    let lexer = Lexer::new("type Query { a a a a a a a a a }").with_limit(26);
    let (tokens, errors) = lexer.lex();
    assert_eq!(tokens.len(), 26);
    assert!(errors.is_empty());

    let lexer = Lexer::new("type Query { a a a a a a a a a }").with_limit(25);
    let (tokens, errors) = lexer.lex();
    assert_eq!(tokens.len(), 25);
    assert_eq!(errors, &[Error::limit("token limit reached, aborting lexing", 31)]);
}

#[test]
fn errors_and_token_limit() {
    let lexer = Lexer::new("type Query { ..a a a a a a a a a }").with_limit(10);
    let (tokens, errors) = lexer.lex();
    // Errors contribute to the token limit.
    assert_eq!(tokens.len(), 9);
    assert_eq!(
        errors,
        &[
            Error::with_loc("Unterminated spread operator", "..".to_string(), 13),
            Error::limit("token limit reached, aborting lexing", 18),
        ],
    );
}

#[test]
fn stream_produces_original_input() {
    let schema = r#"
type Query {
    name: String
    format: String = "Y-m-d\\TH:i:sP"
}
        "#;

    let processed_schema =
        Lexer::new(schema).fold(String::new(), |acc, token| acc + token.unwrap().data());

    assert_eq!(schema, processed_schema);
}

#[test]
fn quoted_block_comment() {
    let input = r#"
"""
Not an escape character:
'/\W/'
Escape character:
\"""
\"""\"""
Not escape characters:
\" \""
Escape character followed by a quote:
\""""
"""
        "#;

    let (tokens, errors) = Lexer::new(input).lex();
    assert!(errors.is_empty());
    // The token data should be literally the source text.
    assert_eq!(
        tokens[1].data(),
        r#"
"""
Not an escape character:
'/\W/'
Escape character:
\"""
\"""\"""
Not escape characters:
\" \""
Escape character followed by a quote:
\""""
"""
"#
        .trim(),
    );

    let input = r#"
# String contents: """
"""\""""""
# Unclosed block string
"""\"""
        "#;
    let (tokens, errors) = Lexer::new(input).lex();
    assert_eq!(tokens[3].data(), r#""""\"""""""#);
    assert_eq!(
        errors,
        &[Error::with_loc(
            "unterminated string value",
            r#""""\"""
        "#
            .to_string(),
            59,
        )]
    );
}

#[test]
fn unexpected_character() {
    let schema = r#"
type Query {
    name: String
}
/
        "#;
    let (_, errors) = Lexer::new(schema).lex();
    assert_eq!(errors, &[Error::with_loc("Unexpected character \"/\"", "/".to_string(), 33)]);
}

#[test]
fn spread_followed_by_multibyte_character() {
    // Previously panicked: the error data sliced inside the multibyte char.
    let (tokens, errors) = Lexer::new(".\u{20AC}").lex();
    assert_eq!(tokens.len(), 1); // Eof
    assert_eq!(
        errors,
        &[Error::with_loc("Unterminated spread operator", ".\u{20AC}".to_string(), 0)]
    );
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
