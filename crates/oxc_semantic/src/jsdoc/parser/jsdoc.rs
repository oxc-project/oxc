use super::jsdoc_parts::JSDocCommentPart;
use super::jsdoc_tag::JSDocTag;
use super::parse::parse_jsdoc;
use oxc_span::Span;
use std::cell::OnceCell;

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
    use crate::{Semantic, SemanticBuilder};
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    fn build_semantic<'a>(allocator: &'a Allocator, source_text: &'a str) -> Semantic<'a> {
        let source_type = SourceType::default();
        let ret = Parser::new(allocator, source_text, source_type).parse();
        let program = allocator.alloc(ret.program);
        let semantic = SemanticBuilder::new(source_text, source_type)
            .with_trivias(ret.trivias)
            .build(program)
            .semantic;
        semantic
    }

    #[test]
    fn jsdoc_span() {
        let allocator = Allocator::default();
        let semantic = build_semantic(
            &allocator,
            r"
            /** single line */
            /**
             * multi
             * line1
             */
            /**
multi
line2
             */
            ",
        );
        let mut jsdocs = semantic.jsdoc().iter_all();

        let jsdoc = jsdocs.next().unwrap();
        assert_eq!(jsdoc.span.source_text(semantic.source_text), " single line ");
        let jsdoc = jsdocs.next().unwrap();
        assert_eq!(
            jsdoc.span.source_text(semantic.source_text),
            "\n             * multi\n             * line1\n             "
        );
        let jsdoc = jsdocs.next().unwrap();
        assert_eq!(jsdoc.span.source_text(semantic.source_text), "\nmulti\nline2\n             ");
    }

    #[test]
    fn jsdoc_comment() {
        let allocator = Allocator::default();
        let semantic = build_semantic(
            &allocator,
            r"
            /** single line @k1 c1 @k2 */
            /**
             * multi
             * line
             * @k1 c1a
             * c1b
             * @k2 c2
             * @k3 c3a
             * c3b
             */
            /** * list */
            ",
        );
        let mut jsdocs = semantic.jsdoc().iter_all();

        let jsdoc = jsdocs.next().unwrap();
        let comment = jsdoc.comment();
        assert_eq!(comment.parsed(), "single line");
        assert_eq!(comment.span.source_text(semantic.source_text), " single line ");

        let jsdoc = jsdocs.next().unwrap();
        let comment = jsdoc.comment();
        assert_eq!(comment.parsed(), "multi\nline");
        assert_eq!(
            comment.span.source_text(semantic.source_text),
            "\n             * multi\n             * line\n             * "
        );

        let jsdoc = jsdocs.next().unwrap();
        let comment = jsdoc.comment();
        assert_eq!(comment.parsed(), "* list");
        assert_eq!(comment.span.source_text(semantic.source_text), " * list ");
    }

    #[test]
    fn jsdoc_tags() {
        let allocator = Allocator::default();
        let semantic = build_semantic(
            &allocator,
            r"
            /** single line @k1 c1 @k2 */
            /**
             * multi
             * line
             * @k1 c1a
             * c1b
             * @k2 c2
             * @k3 c3a
             * c3b
             */
            ",
        );
        let mut jsdocs = semantic.jsdoc().iter_all();

        let jsdoc = jsdocs.next().unwrap();
        assert_eq!(jsdoc.tags().len(), 2);

        let jsdoc = jsdocs.next().unwrap();
        assert_eq!(jsdoc.tags().len(), 3);
    }

    // fn parse_from_full_text(full_text: &str) -> JSDoc {
    //     // Outside of markers can be trimmed
    //     let source_text = full_text.trim().trim_start_matches("/**").trim_end_matches("*/");
    //     let jsdoc = JSDoc::new(source_text, Span::new(3, 3));
    //     jsdoc
    // }

    // #[test]
    // fn parses_jsdoc_comment() {
        // for (full_text, expect_comment) in
        //     [("/**hello*/", "hello"), ("/** hello full_text */", "hello full_text")]
        // {
        //     let jsdoc = parse_from_full_text(full_text);
        //     let comment_part = jsdoc.comment();
        //     assert_eq!(comment_part.parsed(), expect_comment.to_string());
        //     println!("`{}`", comment_part.span.source_text(full_text));
        // }
        //         assert_eq!(
        //             parse_from_full_text("/** hello full_text */"),
        //             ("hello full_text".to_string(), vec![])
        //         );
        //         assert_eq!(parse_from_full_text("/***/"), (String::new(), vec![]));
        //         assert_eq!(parse_from_full_text("/****/"), ("*".to_string(), vec![]));
        //         assert_eq!(parse_from_full_text("/*****/"), ("**".to_string(), vec![]));
        //         assert_eq!(
        //             parse_from_full_text(
        //                 "/**
        //                   * * x
        //                   ** y
        //                   */"
        //             )
        //             .0,
        //             "* x\n* y"
        //         );

        //         assert_eq!(parse_from_full_text("/** <- trim -> */").0, "<- trim ->");
        //         assert_eq!(
        //             parse_from_full_text(
        //                 "
        //         /**
        //          * <- omit this, keep this -> *
        //          */
        //         "
        //             )
        //             .0,
        //             "<- omit this, keep this -> *"
        //         );

        //         assert_eq!(
        //             parse_from_full_text(
        //                 "/**
        // this is
        // comment {@link link} ...
        // @x
        // */"
        //             )
        //             .0,
        //             "this is\ncomment {@link link} ..."
        //         );
        //         assert_eq!(
        //             parse_from_full_text(
        //                 "/**
        // 　　　　　　　　　* 日本語とか
        // 　　　　　　　　　* multibyte文字はどう⁉️
        //                   */"
        //             )
        //             .0,
        //             "日本語とか\nmultibyte文字はどう⁉️"
        //         );

        //         assert_eq!(
        //             parse_from_full_text("/**\nhello {@see inline} source {@a 2}\n*/").0,
        //             "hello {@see inline} source {@a 2}"
        //         );

        //         assert_eq!(parse_from_full_text("/** ハロー @comment だよ*/").0, "ハロー");
    // }

    //     #[test]
    //     fn parses_jsdoc_tags() {
    //         assert_eq!(
    //             parse_from_full_text("/**@deprecated*/").1,
    //             vec![parse_jsdoc_tag("@deprecated")]
    //         );
    //         assert_eq!(
    //             parse_from_full_text("/**@foo since 2024 */").1,
    //             vec![parse_jsdoc_tag("@foo since 2024 ")]
    //         );

    //         assert_eq!(
    //             parse_from_full_text("/** @foo @bar */").1,
    //             vec![parse_jsdoc_tag("@foo "), parse_jsdoc_tag("@bar ")]
    //         );

    //         assert_eq!(parse_from_full_text("/**@*/").1, vec![parse_jsdoc_tag("@")]);

    //         assert_eq!(
    //             parse_from_full_text("/** @aiue あいうえ @o お*/").1,
    //             vec![parse_jsdoc_tag("@aiue あいうえ "), parse_jsdoc_tag("@o お")],
    //         );
    //         assert_eq!(
    //             parse_from_full_text("/** @a @@ @d */").1,
    //             vec![
    //                 parse_jsdoc_tag("@a "),
    //                 parse_jsdoc_tag("@"),
    //                 parse_jsdoc_tag("@ "),
    //                 parse_jsdoc_tag("@d ")
    //             ],
    //         );

    //         assert_eq!(
    //             parse_from_full_text(
    //                 "/** @yo
    //     */"
    //             )
    //             .1,
    //             vec![parse_jsdoc_tag("@yo\n    ")]
    //         );
    //         assert_eq!(
    //             parse_from_full_text(
    //                 "/**
    //                     *     @foo
    //                           */"
    //             )
    //             .1,
    //             vec![parse_jsdoc_tag("@foo\n                          ")]
    //         );
    //         assert_eq!(
    //             parse_from_full_text(
    //                 "
    //         /**
    //          * @x with asterisk
    //          */
    //                 "
    //             )
    //             .1,
    //             vec![parse_jsdoc_tag("@x with asterisk\n         ")]
    //         );
    //         assert_eq!(
    //             parse_from_full_text(
    //                 "
    //             /**
    //             @y without
    //         asterisk
    //              */
    //                     "
    //             )
    //             .1,
    //             vec![parse_jsdoc_tag("@y without\n        asterisk\n             ")]
    //         );

    //         assert_eq!(
    //             parse_from_full_text(
    //                 "
    //     /**
    //        @foo@bar
    //     * @baz
    //      */
    //             "
    //             )
    //             .1,
    //             vec![
    //                 parse_jsdoc_tag("@foo"),
    //                 parse_jsdoc_tag("@bar\n    * "),
    //                 parse_jsdoc_tag("@baz\n     ")
    //             ]
    //         );
    //         assert_eq!(
    //             parse_from_full_text(
    //                 "/**
    //                       * @one
    //                   *
    //                   * ...
    //               *
    //                       * @two */"
    //             )
    //             .1,
    //             vec![
    //                 parse_jsdoc_tag("@one\n                  *\n                  * ...\n              *\n                      * "),
    //                 parse_jsdoc_tag("@two ")
    //             ]
    //         );
    //         assert_eq!(
    //             parse_from_full_text(
    //                 "/**
    //                   * ...
    //                   * @hey you!
    //                   *   Are you OK?
    //                   * @yes I'm fine
    //                   */"
    //             )
    //             .1,
    //             vec![
    //                 parse_jsdoc_tag(
    //                     "@hey you!\n                  *   Are you OK?\n                  * "
    //                 ),
    //                 parse_jsdoc_tag("@yes I'm fine\n                  ")
    //             ]
    //         );
    //     }

    //     #[test]
    //     fn parses_practical() {
    //         let jsdoc = parse_from_full_text(
    //             "
    // /**
    //  * @typedef {Object} User - a User account
    //  * @property {string} displayName - the name used to show the user
    //  * @property {number} id - a unique id
    //  */
    // ",
    //         );
    //         let mut tags = jsdoc.1.iter();
    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "typedef");
    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "property");
    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "property");

    //         let jsdoc = parse_from_full_text(
    //             "
    // /**
    //  * Adds two numbers together
    //  * @param {number} a The first number
    //  * @param {number} b The second number
    //  * @returns {number}
    //  */
    // ",
    //         );
    //         let mut tags = jsdoc.1.iter();
    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "param");
    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "param");
    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "returns");
    //     }

    //     #[test]
    //     fn parses_practical_with_multibyte() {
    //         let jsdoc = parse_from_full_text(
    //             "/**
    //               * flat tree data on expanded state
    //               *
    //               * @export
    //               * @template T
    //               * @param {*} data : table data
    //               * @param {string} childrenColumnName : 指定树形结构的列名
    //               * @param {Set<Key>} expandedKeys : 展开的行对应的keys
    //               * @param {GetRowKey<T>} getRowKey  : 获取当前rowKey的方法
    //               * @returns flattened data
    //               */",
    //         );
    //         assert_eq!(jsdoc.0, "flat tree data on expanded state");
    //         let mut tags = jsdoc.1.iter();
    //         assert_eq!(tags.len(), 7);

    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "export");
    //         assert_eq!(tag.comment(), "");

    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "template");
    //         assert_eq!(tag.comment(), "T");

    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "param");
    //         assert_eq!(tag.type_name_comment(), (Some("*"), Some("data"), ": table data".to_string()));

    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "param");
    //         assert_eq!(
    //             tag.type_name_comment(),
    //             (Some("string"), Some("childrenColumnName"), ": 指定树形结构的列名".to_string())
    //         );

    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "param");
    //         assert_eq!(
    //             tag.type_name_comment(),
    //             (Some("Set<Key>"), Some("expandedKeys"), ": 展开的行对应的keys".to_string())
    //         );

    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "param");
    //         assert_eq!(
    //             tag.type_name_comment(),
    //             (Some("GetRowKey<T>"), Some("getRowKey"), ": 获取当前rowKey的方法".to_string())
    //         );

    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "returns");
    //         assert_eq!(tag.type_comment(), (None, "flattened data".to_string()));
    //     }
}
