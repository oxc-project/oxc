#[macro_export]
macro_rules! ast_helper {
    ($allocator: expr, $source: ident) => {{
        let source_type = oxc_span::SourceType::mjs();
        let ret = oxc_parser::Parser::new($allocator, &$source, source_type).parse();
        if !ret.errors.is_empty() {
            // Since the code is only used in `oxc_transformer` crate, we can use `panic!` here.
            panic!("Parser Errors: {:?} in using ast_helper", ret.errors);
        }
        ret
    }};
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;

    #[test]
    fn test_ast_helper() {
        let allocator = Allocator::default();
        let source = format!("const a = {}", 0);
        let ast = ast_helper!(&allocator, source);
        assert_eq!(ast.program.body.len(), 1);
    }

    #[test]
    fn test_more_complex_ast_helper() {
        let allocator = Allocator::default();
        let key = "key";
        let src = format!(
            r#"Object.defineProperty(exports, {key}, {{
    enumerable: true,
    writable: true,
    value: function () {{
        return 0;
    }}
}})
"#
        );
        let ast = ast_helper!(&allocator, src);
        assert_eq!(ast.program.body.len(), 1);
    }
}
