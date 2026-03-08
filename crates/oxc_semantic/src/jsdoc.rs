use rustc_hash::FxHashMap;

use oxc_jsdoc::JSDoc;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, AstNodes};

#[derive(Debug, Default)]
pub struct JSDocFinder<'a> {
    /// JSDocs by Span
    attached: FxHashMap<u32, Vec<JSDoc<'a>>>,
    not_attached: Vec<JSDoc<'a>>,
}

impl<'a> JSDocFinder<'a> {
    pub fn new(attached: FxHashMap<u32, Vec<JSDoc<'a>>>, not_attached: Vec<JSDoc<'a>>) -> Self {
        Self { attached, not_attached }
    }

    pub fn get_one_by_node<'b>(
        &'b self,
        nodes: &AstNodes<'a>,
        node: &AstNode<'a>,
    ) -> Option<JSDoc<'a>> {
        let jsdocs = self.get_all_by_node(nodes, node)?;

        // If flagged, at least 1 JSDoc is attached
        // If multiple JSDocs are attached, return the last = nearest
        jsdocs.last().cloned()
    }

    pub fn get_all_by_node<'b>(
        &'b self,
        nodes: &AstNodes<'a>,
        node: &AstNode<'a>,
    ) -> Option<Vec<JSDoc<'a>>> {
        if !nodes.flags(node.id()).has_jsdoc() {
            return None;
        }

        let span = node.kind().span();
        self.get_all_by_span(span)
    }

    pub fn get_all_by_span<'b>(&'b self, span: Span) -> Option<Vec<JSDoc<'a>>> {
        self.attached.get(&span.start).cloned()
    }

    pub fn iter_all<'b>(&'b self) -> impl Iterator<Item = &'b JSDoc<'a>> + 'b {
        self.attached.values().flatten().chain(self.not_attached.iter())
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_jsdoc::JSDoc;
    use oxc_parser::Parser;
    use oxc_span::{SourceType, Span};

    use crate::{Semantic, SemanticBuilder};

    fn build_semantic_default<'a>(allocator: &'a Allocator, source_text: &'a str) -> Semantic<'a> {
        build_semantic(allocator, source_text, None)
    }

    fn build_semantic<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: Option<SourceType>,
    ) -> Semantic<'a> {
        let source_type = source_type.unwrap_or_default();
        let ret = Parser::new(allocator, source_text, source_type).parse();
        SemanticBuilder::new().build(allocator.alloc(ret.program)).semantic
    }

    fn get_jsdocs<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        symbol: &'a str,
        source_type: Option<SourceType>,
    ) -> Option<Vec<JSDoc<'a>>> {
        let semantic = build_semantic(allocator, source_text, source_type);
        let start = u32::try_from(source_text.find(symbol).unwrap_or(0)).unwrap();
        let span = Span::sized(start, u32::try_from(symbol.len()).unwrap());
        semantic.jsdoc().get_all_by_span(span)
    }

    fn test_jsdoc_found(source_text: &str, symbol: &str, source_type: Option<SourceType>) {
        let allocator = Allocator::default();
        assert!(
            get_jsdocs(&allocator, source_text, symbol, source_type).is_some(),
            "JSDoc should found for\n  {symbol} \nin\n  {source_text}"
        );
    }

    fn test_jsdoc_not_found(source_text: &str, symbol: &str) {
        let allocator = Allocator::default();
        assert!(
            get_jsdocs(&allocator, source_text, symbol, None).is_none(),
            "JSDoc should NOT found for\n  {symbol} \nin\n  {source_text}"
        );
    }

    #[test]
    fn not_found() {
        let source_texts = [
            ("function f1() {}", "function f1() {}"),
            ("// test", "function f2() {}"),
            ("/* test */function f3() {}", "function f3() {}"),
            ("/** for 4a */ ; function f4a() {} function f4b() {}", "function f4b() {}"),
            ("function f4a() {} /** for 4b */ ; function f4b() {} ", "function f4a() {}"),
            ("function f5() {} /** test */", "function f5() {}"),
            (
                "/** for o */
                const o = {
                    f6() {}
                };",
                "f6() {}",
            ),
            ("/** for () */ (() => {})", "() => {}"),
            ("/** for ; */ ; let v1;", "let v1;"),
            ("/** for let v2 */ let v2 = () => {};", "() => {}"),
            ("/** for if */ if (true) { let v3; })", "let v3;"),
            (
                "class C1 {
                    /** for m1 */
                    m1() {}
                    m2() {}
                }",
                "m2() {}",
            ),
        ];
        for (source_text, target) in source_texts {
            test_jsdoc_not_found(source_text, target);
        }
    }

    #[test]
    fn found() {
        let source_texts = [
            ("/** test */function f1() {}", "function f1() {}"),
            ("/*** test */function f2() {}", "function f2() {}"),
            (
                "
            /** test */
        function f3() {}",
                "function f3() {}",
            ),
            (
                "/** test */


                function f4() {}",
                "function f4() {}",
            ),
            (
                "/**
             * test
             * */
            function f5() {}",
                "function f5() {}",
            ),
            (
                "/** test */
                // ---
                function f6() {}",
                "function f6() {}",
            ),
            (
                "/** test */
                /* -- */
                function f7() {}",
                "function f7() {}",
            ),
            (
                "/** test */
                /** test2 */
                function f8() {}",
                "function f8() {}",
            ),
            (
                "/** test */ /** test2 */
                function f9() {}",
                "function f9() {}",
            ),
            (
                "/** for f10 */ function f10() {} /** for f11 */ function f11() {}",
                "function f11() {}",
            ),
            (
                "const o = {
                    /** for f12 */
                    f12() {}
                };",
                "f12() {}",
            ),
            ("/** test */ (() => {})", "(() => {})"),
            ("/** test */ let v1 = 1", "let v1 = 1"),
            ("let v2a = 1, /** for v2b */ v2b = 2", "v2b = 2"),
            ("/** for v3a */ const v3a = 1, v3b = 2;", "const v3a = 1, v3b = 2;"),
            ("/** test */ export const e1 = 1;", "export const e1 = 1;"),
            ("/** test */ export default {};", "export default {};"),
            ("/** test */ import 'i1'", "import 'i1'"),
            ("/** test */ import I from 'i2'", "import I from 'i2'"),
            ("/** test */ import { I } from 'i3'", "import { I } from 'i3'"),
        ];
        for (source_text, target) in source_texts {
            test_jsdoc_found(source_text, target, None);
        }
    }

    #[test]
    fn found_ts() {
        let source_texts = [(
            "class Foo {
            /** jsdoc */
            bar: string;
        }",
            "bar: string;",
        )];

        let source_type = SourceType::default().with_typescript(true);
        for (source_text, target) in source_texts {
            test_jsdoc_found(source_text, target, Some(source_type));
        }
    }

    #[test]
    fn get_all_by_span_order() {
        let allocator = Allocator::default();
        let source_text = r"
            /**c0*/
            function foo() {}

            /**c1*/
            /* noop */
            /**c2*/
            // noop
            /**c3*/
            const x = () => {};
        ";
        let symbol = "const x = () => {};";
        let jsdocs = get_jsdocs(&allocator, source_text, symbol, None);

        assert!(jsdocs.is_some());
        let jsdocs = jsdocs.unwrap();
        assert_eq!(jsdocs.len(), 3);

        // Should be [farthest, ..., nearest]
        let mut iter = jsdocs.iter();
        let c1 = iter.next().unwrap();
        assert_eq!(c1.comment().parsed(), "c1");
        let c2 = iter.next().unwrap();
        assert_eq!(c2.comment().parsed(), "c2");
        let c3 = iter.next().unwrap();
        assert_eq!(c3.comment().parsed(), "c3");
    }

    #[test]
    fn get_all_jsdoc() {
        let allocator = Allocator::default();
        let semantic = build_semantic(
            &allocator,
            r"
            // noop
            /** 1. ; */
            // noop
            ;
            /** 2. class X {} *//** 3. class X {} */
            class X {
                /** 4. foo */
                foo = /** 5. () */ (() => {});
            }

            /** 6. let x; */
            /* noop */

            let x;

            /**/ // noop and noop

            /** 7. Not attached but collected! */
            ",
            Some(SourceType::default()),
        );
        assert_eq!(semantic.jsdoc().iter_all().count(), 7);
    }

    // Tests from jsdoc/parser/jsdoc.rs

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
            let semantic = build_semantic_default(&allocator, source_text);
            let mut jsdocs = semantic.jsdoc().iter_all();

            let jsdoc = jsdocs.next().unwrap();
            assert_eq!(jsdoc.span.source_text(source_text), span_text);
        }
    }

    #[test]
    fn jsdoc_comment() {
        for (source_text, parsed, span_text, tag_len) in [
            (
                "/** single line @k1 c1 @k2 */",
                "single line @k1 c1 @k2",
                " single line @k1 c1 @k2 ",
                0,
            ),
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
                "ハロー @comment だよ",
                " ハロー @comment だよ",
                0,
            ),
        ] {
            let allocator = Allocator::default();
            let semantic = build_semantic_default(&allocator, source_text);
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
        let semantic = build_semantic_default(
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
        let semantic = build_semantic_default(
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
        let semantic = build_semantic_default(
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

    #[test]
    fn parses_issue_11992() {
        use oxc_jsdoc::parser::jsdoc_parts::JSDocCommentPart;
        let allocator = Allocator::default();
        let semantic = build_semantic_default(
            &allocator,
            "/**@property [
*/",
        );
        let jsdoc = semantic.jsdoc().iter_all().next().unwrap();

        let mut tags = jsdoc.tags().iter();
        assert_eq!(tags.len(), 1);

        let tag = tags.next().unwrap();
        assert_eq!(
            tag.type_name_comment(),
            (None, None, JSDocCommentPart::new(" [\n", Span::new(12, 15)))
        );
        assert_eq!(tag.kind.parsed(), "property");
    }

    // Tests from jsdoc/parser/jsdoc_tag.rs

    #[test]
    fn jsdoc_tag_span() {
        for (source_text, tag_span_text) in [
            (
                "
                /**
                 * @k2 c2a
                 * c2b
                 *
                 */
                ",
                "@k2 c2a\n                 * c2b\n                 *\n                 ",
            ),
            (
                "
                /**
                 * multi
                 * @k3 c3
                 */
                ",
                "@k3 c3\n                 ",
            ),
            (
                "
                /**
                 * Has single quote ' in comment
                 * @k5 c5
                 */",
                "@k5 c5\n                 ",
            ),
            (
                "
                /**
                 * @import {T} from '@k6'
                 */",
                "@import {T} from '@k6'\n                 ",
            ),
        ] {
            let allocator = Allocator::default();
            let semantic = build_semantic_default(&allocator, source_text);
            let mut jsdocs = semantic.jsdoc().iter_all();

            let tag = jsdocs.next().unwrap().tags().first().unwrap();
            assert_eq!(tag.span.source_text(source_text), tag_span_text);
        }
    }

    #[test]
    fn jsdoc_tag_kind() {
        for (source_text, tag_kind, tag_kind_span_text) in [
            (
                "/**
             * multi
             * line
             * @k3 c3a
             * c3b
             */",
                "k3",
                "@k3",
            ),
            (" /**@*/ ", "", "@"),
            (" /**@@*/ ", "@", "@@"),
            (" /** @あいう え */ ", "あいう", "@あいう"),
        ] {
            let allocator = Allocator::default();
            let semantic = build_semantic_default(&allocator, source_text);
            let mut jsdocs = semantic.jsdoc().iter_all();

            let tag = jsdocs.next().unwrap().tags().first().unwrap();
            assert_eq!(tag.kind.parsed(), tag_kind);
            assert_eq!(tag.kind.span.source_text(source_text), tag_kind_span_text);
        }
    }

    #[test]
    fn jsdoc_tag_comment() {
        for (source_text, parsed_comment_part) in [
            (
                "/**
             * multi
             * line
             * @k3 c3a
             * c3b
             */",
                ("c3a\nc3b", " c3a\n             * c3b\n             "),
            ),
            ("/**@k5 c5 w/ {@inline}!*/", ("c5 w/ {@inline}!", " c5 w/ {@inline}!")),
            (" /**@k6 */ ", ("", " ")),
            (" /**@*/ ", ("", "")),
            (" /**@@*/ ", ("", "")),
            (" /** @あいう え */ ", ("え", " え ")),
        ] {
            let allocator = Allocator::default();
            let semantic = build_semantic_default(&allocator, source_text);
            let mut jsdocs = semantic.jsdoc().iter_all();

            let comment = jsdocs.next().unwrap().tags().first().unwrap().comment();
            assert_eq!(
                (comment.parsed().as_str(), comment.span.source_text(source_text)),
                parsed_comment_part
            );
        }
    }

    #[test]
    fn jsdoc_tag_type() {
        for (source_text, parsed_type_part) in [
            ("/** @k0 */", None),
            ("/** @k1 {t1} */", Some(("t1", "{t1}"))),
            ("/** @k1 {} */", Some(("", "{}"))),
            (
                "/** @k2
            {t2} */",
                Some(("t2", "{t2}")),
            ),
            ("/** @k3 { t3  } */", Some(("t3", "{ t3  }"))),
            ("/** @k4 x{t4}y */", None),
            ("/** @k5 {t5}} */", Some(("t5", "{t5}"))),
            ("/** @k6  */", None),
            ("/** @k7 x */", None),
            ("/** @k8 { */", None),
            ("/** @k9 {t9 */", None),
            ("/** @k10 {{t10} */", None),
        ] {
            let allocator = Allocator::default();
            let semantic = build_semantic_default(&allocator, source_text);
            let mut jsdocs = semantic.jsdoc().iter_all();

            let type_part = jsdocs.next().unwrap().tags().first().unwrap().r#type();
            assert_eq!(
                type_part.map(|t| (t.parsed(), t.span.source_text(source_text))),
                parsed_type_part
            );
        }
    }

    #[test]
    fn jsdoc_tag_type_comment() {
        for (source_text, parsed_type_part, parsed_comment_part) in [
            ("/** @k */", None, ("", " ")),
            ("/** @k1 {t1} c1 */", Some(("t1", "{t1}")), ("c1", " c1 ")),
            (
                "/** @k2
{t2} */",
                Some(("t2", "{t2}")),
                ("", " "),
            ),
            ("/** @k3  c3 */", None, ("c3", "  c3 ")),
            ("/** @k4\nc4 foo */", None, ("c4 foo", "\nc4 foo ")),
            (
                "/** @k5
{t5}
c5 */",
                Some(("t5", "{t5}")),
                ("c5", "\nc5 "),
            ),
            ("/** @k6 {t6} - c6 */", Some(("t6", "{t6}")), ("- c6", " - c6 ")),
        ] {
            let allocator = Allocator::default();
            let semantic = build_semantic_default(&allocator, source_text);
            let mut jsdocs = semantic.jsdoc().iter_all();

            let (type_part, comment_part) =
                jsdocs.next().unwrap().tags().first().unwrap().type_comment();
            assert_eq!(
                type_part.map(|t| (t.parsed(), t.span.source_text(source_text))),
                parsed_type_part
            );
            assert_eq!(
                (comment_part.parsed().as_str(), comment_part.span.source_text(source_text)),
                parsed_comment_part
            );
        }
    }

    #[test]
    fn jsdoc_tag_type_name_comment() {
        for (source_text, parsed_type_part, parsed_type_name_part, parsed_comment_part) in [
            ("/** @k */", None, None, ("", " ")),
            ("/** @k\n\n*/", None, None, ("", "\n\n")),
            ("/** @k1 {t1} n1 c1 */", Some(("t1", "{t1}")), Some(("n1", "n1")), ("c1", " c1 ")),
            ("/** @k2 {t2} n2*/", Some(("t2", "{t2}")), Some(("n2", "n2")), ("", "")),
            ("/** @k3 n3 c3 */", None, Some(("n3", "n3")), ("c3", " c3 ")),
            (
                "/** @k4 n4 c4
...*/",
                None,
                Some(("n4", "n4")),
                ("c4\n...", " c4\n..."),
            ),
            (
                "/** @k5  {t5}  n5  - c5 */",
                Some(("t5", "{t5}")),
                Some(("n5", "n5")),
                ("- c5", "  - c5 "),
            ),
            (
                "/** @k6
{t6}
n6
c6 */",
                Some(("t6", "{t6}")),
                Some(("n6", "n6")),
                ("c6", "\nc6 "),
            ),
            (
                "/** @k7

{t7}

n7

c7 */",
                Some(("t7", "{t7}")),
                Some(("n7", "n7")),
                ("c7", "\n\nc7 "),
            ),
            ("/** @k8 {t8} */", Some(("t8", "{t8}")), None, ("", " ")),
            ("/** @k9 n9 */", None, Some(("n9", "n9")), ("", " ")),
            ("/** @property n[].n10 */", None, Some(("n[].n10", "n[].n10")), ("", " ")),
            ("/** @property n.n11 */", None, Some(("n.n11", "n.n11")), ("", " ")),
            (
                r#"/** @property [cfg.n12="default value"] */"#,
                None,
                Some(("cfg.n12", r#"[cfg.n12="default value"]"#)),
                ("", " "),
            ),
            (
                "/** @property {t13} [n = 13] c13 */",
                Some(("t13", "{t13}")),
                Some(("n", "[n = 13]")),
                ("c13", " c13 "),
            ),
            (
                "/** @param {t14} [n14] - opt */",
                Some(("t14", "{t14}")),
                Some(("n14", "[n14]")),
                ("- opt", " - opt "),
            ),
            ("/** @param {t15}a */", Some(("t15", "{t15}")), Some(("a", "a")), ("", " ")),
            ("/** @type{t16}n16*/", Some(("t16", "{t16}")), Some(("n16", "n16")), ("", "")),
            (
                "/** @param entries Entries in the {@link SearchableMap} */",
                None,
                Some(("entries", "entries")),
                ("Entries in the {@link SearchableMap}", " Entries in the {@link SearchableMap} "),
            ),
            (
                "/** @param bar - With braces {} */",
                None,
                Some(("bar", "bar")),
                ("- With braces {}", " - With braces {} "),
            ),
            (
                "/** @param {string} name See {@link Foo} */",
                Some(("string", "{string}")),
                Some(("name", "name")),
                ("See {@link Foo}", " See {@link Foo} "),
            ),
        ] {
            let allocator = Allocator::default();
            let semantic = build_semantic_default(&allocator, source_text);
            let mut jsdocs = semantic.jsdoc().iter_all();

            let (type_part, type_name_part, comment_part) =
                jsdocs.next().unwrap().tags().first().unwrap().type_name_comment();
            assert_eq!(
                type_part.map(|t| (t.parsed(), t.span.source_text(source_text))),
                parsed_type_part,
                "type_part failed to assert in {source_text}"
            );
            assert_eq!(
                type_name_part.map(|n| (n.parsed(), n.span.source_text(source_text))),
                parsed_type_name_part,
                "type_name_part failed to assert in {source_text}"
            );
            assert_eq!(
                (comment_part.parsed().as_str(), comment_part.span.source_text(source_text)),
                parsed_comment_part,
                "comment_part failed to assert in {source_text}"
            );
        }
    }
}
