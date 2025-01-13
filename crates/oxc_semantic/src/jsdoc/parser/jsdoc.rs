use std::cell::OnceCell;

use oxc_span::Span;

use super::{jsdoc_parts::JSDocCommentPart, jsdoc_tag::JSDocTag, parse::parse_jsdoc};

type ParsedJSDoc<'a> = (JSDocCommentPart<'a>, Vec<JSDocTag<'a>>);

#[derive(Debug, Clone)]
pub struct JSDoc<'a> {
    raw: &'a str,
    /// Cached+parsed JSDoc comment and tags
    cached: OnceCell<ParsedJSDoc<'a>>,
    pub span: Span,
}

impl<'a> JSDoc<'a> {
    /// comment_content: Inside of /**HERE*/, not include `/**` and `*/`
    /// span: `Span` for this JSDoc comment, range for `/**HERE*/`
    pub fn new(comment_content: &'a str, span: Span) -> JSDoc<'a> {
        Self { raw: comment_content, cached: OnceCell::new(), span }
    }

    pub fn comment(&self) -> JSDocCommentPart<'a> {
        self.parse().0
    }

    pub fn tags(&self) -> &Vec<JSDocTag<'a>> {
        &self.parse().1
    }

    fn parse(&self) -> &ParsedJSDoc<'a> {
        self.cached.get_or_init(|| parse_jsdoc(self.raw, self.span.start))
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    use crate::{Semantic, SemanticBuilder};

    fn build_semantic<'a>(allocator: &'a Allocator, source_text: &'a str) -> Semantic<'a> {
        let source_type = SourceType::default();
        let ret = Parser::new(allocator, source_text, source_type).parse();
        SemanticBuilder::new().with_build_jsdoc(true).build(&ret.program).semantic
    }

    #[test]
    fn jsdoc_span() {
        for (source_text, span_text) in [
            ("/** single line */", " single line "),
            (
                "
            /**
             * multi
             * line1
             */",
                "\n             * multi\n             * line1\n             ",
            ),
            (
                "
            /**
multi
line2
             */
",
                "\nmulti\nline2\n             ",
            ),
        ] {
            let allocator = Allocator::default();
            let semantic = build_semantic(&allocator, source_text);
            let mut jsdocs = semantic.jsdoc().iter_all();

            let jsdoc = jsdocs.next().unwrap();
            assert_eq!(jsdoc.span.source_text(source_text), span_text);
        }
    }

    #[test]
    fn jsdoc_comment() {
        for (source_text, parsed, span_text, tag_len) in [
            ("/** single line @k1 c1 @k2 */", "single line", " single line ", 2),
            (
                "/**
             * multi
             * line
             * @k1 c1a
             * c1b
             * @k2 c2
             * @k3 c3a
             * c3b
             */",
                "multi\nline",
                "\n             * multi\n             * line\n             * ",
                3,
            ),
            (" /** * list */ ", "* list", " * list ", 0),
            ("/***/", "", "", 0),
            ("/****/", "*", "*", 0),
            ("/*****/", "**", "**", 0),
            (
                "
                    /**
                      * * x
                      ** y
                      */
",
                "* x\n* y",
                "\n                      * * x\n                      ** y\n                      ",
                0,
            ),
            (
                "
    /** <- trim -> */
",
                "<- trim ->",
                " <- trim -> ",
                0,
            ),
            (
                "
             /**
              * <- omit this, keep this -> *
              */
",
                "<- omit this, keep this -> *",
                "\n              * <- omit this, keep this -> *\n              ",
                0,
            ),
            (
                "
        /**
        this is
        comment {@link link} ...
        @x
        */
",
                "this is\ncomment {@link link} ...",
                "\n        this is\n        comment {@link link} ...\n        ",
                1,
            ),
            (
                "
/**
 * 日本語とか
 * multibyte文字はどう⁉️
 */
",
                "日本語とか\nmultibyte文字はどう⁉️",
                "\n * 日本語とか\n * multibyte文字はどう⁉️\n ",
                0,
            ),
            (
                "
/**\nhello {@see inline} source {@a 2}\n*/
",
                "hello {@see inline} source {@a 2}",
                "\nhello {@see inline} source {@a 2}\n",
                0,
            ),
            (
                "
    /** ハロー @comment だよ*/
",
                "ハロー",
                " ハロー ",
                1,
            ),
        ] {
            let allocator = Allocator::default();
            let semantic = build_semantic(&allocator, source_text);
            let mut jsdocs = semantic.jsdoc().iter_all();

            let jsdoc = jsdocs.next().unwrap();
            let (comment, tags) = (jsdoc.comment(), jsdoc.tags());
            assert_eq!(comment.parsed(), parsed);
            assert_eq!(comment.span.source_text(source_text), span_text);
            assert_eq!(tags.len(), tag_len);
        }
    }

    #[test]
    fn parses_practical() {
        let allocator = Allocator::default();
        let semantic = build_semantic(
            &allocator,
            "/**
              * @typedef {Object} User - a User account
              * @property {string} displayName - the name used to show the user
              * @property {number} id - a unique id
              */
            ",
        );
        let jsdoc = semantic.jsdoc().iter_all().next().unwrap();

        assert_eq!(jsdoc.comment().parsed(), "");
        let mut tags = jsdoc.tags().iter();
        assert_eq!(tags.len(), 3);

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind.parsed(), "typedef");
        let (type_part, name_part, comment_part) = tag.type_name_comment();
        let (type_part, name_part) = (type_part.unwrap(), name_part.unwrap());
        assert_eq!(
            (type_part.parsed(), name_part.parsed(), comment_part.parsed()),
            ("Object", "User", "- a User account".to_string())
        );

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind.parsed(), "property");
        let (type_part, name_part, comment_part) = tag.type_name_comment();
        let (type_part, name_part) = (type_part.unwrap(), name_part.unwrap());
        assert_eq!(
            (type_part.parsed(), name_part.parsed(), comment_part.parsed()),
            ("string", "displayName", "- the name used to show the user".to_string())
        );

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind.parsed(), "property");
        let (type_part, name_part, comment_part) = tag.type_name_comment();
        let (type_part, name_part) = (type_part.unwrap(), name_part.unwrap());
        assert_eq!(
            (type_part.parsed(), name_part.parsed(), comment_part.parsed()),
            ("number", "id", "- a unique id".to_string())
        );
    }

    #[test]
    fn parses_practical_with_multibyte() {
        let allocator = Allocator::default();
        let semantic = build_semantic(
            &allocator,
            "/**
                  * flat tree data on expanded state
                  *
                  * @export
                  * @template T
                  * @param {*} data : table data
                  * @param {string} childrenColumnName : 指定树形结构的列名
                  * @param {Set<Key>} expandedKeys : 展开的行对应的keys
                  * @param {GetRowKey<T>} getRowKey  : 获取当前rowKey的方法
                  * @returns flattened data
                  */",
        );
        let jsdoc = semantic.jsdoc().iter_all().next().unwrap();

        assert_eq!(jsdoc.comment().parsed(), "flat tree data on expanded state");
        let mut tags = jsdoc.tags().iter();
        assert_eq!(tags.len(), 7);

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind.parsed(), "export");
        assert_eq!(tag.comment().parsed(), "");

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind.parsed(), "template");
        assert_eq!(tag.comment().parsed(), "T");

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind.parsed(), "param");
        let (type_part, name_part, comment_part) = tag.type_name_comment();
        let (type_part, name_part) = (type_part.unwrap(), name_part.unwrap());
        assert_eq!(
            (type_part.parsed(), name_part.parsed(), comment_part.parsed()),
            ("*", "data", ": table data".to_string())
        );

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind.parsed(), "param");
        let (type_part, name_part, comment_part) = tag.type_name_comment();
        let (type_part, name_part) = (type_part.unwrap(), name_part.unwrap());
        assert_eq!(
            (type_part.parsed(), name_part.parsed(), comment_part.parsed()),
            ("string", "childrenColumnName", ": 指定树形结构的列名".to_string())
        );

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind.parsed(), "param");
        let (type_part, name_part, comment_part) = tag.type_name_comment();
        let (type_part, name_part) = (type_part.unwrap(), name_part.unwrap());
        assert_eq!(
            (type_part.parsed(), name_part.parsed(), comment_part.parsed()),
            ("Set<Key>", "expandedKeys", ": 展开的行对应的keys".to_string())
        );

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind.parsed(), "param");
        let (type_part, name_part, comment_part) = tag.type_name_comment();
        let (type_part, name_part) = (type_part.unwrap(), name_part.unwrap());
        assert_eq!(
            (type_part.parsed(), name_part.parsed(), comment_part.parsed()),
            ("GetRowKey<T>", "getRowKey", ": 获取当前rowKey的方法".to_string())
        );

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind.parsed(), "returns");
        let (type_part, comment_part) = tag.type_comment();
        assert_eq!((type_part, comment_part.parsed()), (None, "flattened data".to_string()));
    }

    #[test]
    fn parses_with_backticks() {
        let allocator = Allocator::default();
        let semantic = build_semantic(
            &allocator,
            "
            /**
             * This is normal comment, `@xxx` should not parsed as tag.
             *
             * @example ```ts
                // @comment
                @decoratorInComment
                class Foo { }
               ```
             */
            ",
        );
        let jsdoc = semantic.jsdoc().iter_all().next().unwrap();

        let mut tags = jsdoc.tags().iter();
        assert_eq!(tags.len(), 1);

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind.parsed(), "example");
    }
}
