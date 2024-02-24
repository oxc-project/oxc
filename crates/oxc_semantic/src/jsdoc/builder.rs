use std::collections::BTreeMap;
use std::rc::Rc;

use oxc_ast::{AstKind, TriviasMap};
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use super::{JSDoc, JSDocComment};

pub struct JSDocBuilder<'a> {
    source_text: &'a str,
    trivias: Rc<TriviasMap>,
    docs: BTreeMap<Span, Vec<JSDocComment<'a>>>,
    leading_comments_seen: FxHashSet<u32>,
}

impl<'a> JSDocBuilder<'a> {
    pub fn new(source_text: &'a str, trivias: &Rc<TriviasMap>) -> Self {
        Self {
            source_text,
            trivias: Rc::clone(trivias),
            docs: BTreeMap::default(),
            leading_comments_seen: FxHashSet::default(),
        }
    }

    pub fn build(self) -> JSDoc<'a> {
        // TODO: Correct JSDoc for EOF
        JSDoc::new(self.docs)
    }

    // この紐づけ処理は、Semanticのビルドにあわせて行われる。
    // つまり、各ユースケース（たとえばoxlintにおける各ルールの実行）よりも、"前"のタイミング。
    //
    // もしルール側の設定などで、動的にこのロジックを変えたい場合、
    // - そもそもここで事前に紐づけるのを諦め、各ルール側で毎回探すようにするか
    // - ここでより広く緩く（パフォーマンス影響があるため慎重に）紐づけておき、あとから絞り込めるようにする
    // このいずれかが必要になる。
    //
    // そして、「どのノードに対して、JSDocをアタッチするか？」について、特定のSpecがあるわけではない。
    // それぞれの実装が、それぞれのユースケースにあわせて決めている。
    // たとえば、TypeScriptの場合、その対象はかなり広く定義されている。
    // > https://github.com/microsoft/TypeScript/blob/d04e3489b0d8e6bc9a8a9396a633632a5a467328/src/compiler/utilities.ts#L4195
    // `eslint-plugin-jsdoc`の場合、その対象はルールの設定で自由に変えることができる。（デフォルトは関数に関するもののみ）
    // > https://github.com/gajus/eslint-plugin-jsdoc/blob/e948bee821e964a92fbabc01574eca226e9e1252/src/iterateJsdoc.js#L2517-L2536
    //
    // また、そのノードに対して「どのようにJSDocをアタッチするか？」についても、同様に実装依存である。
    // TypeScriptの場合（特定のASTノード、および`endOfFileToken`が、`jsDoc`プロパティを保持）、複数の`JSDocComment`がアタッチされる。
    // `eslint-plugin-jsdoc`の場合、`jsdoccomment`というライブラリが、一部の例外処理と共に、単一のコメントのみを取得している。
    // > https://github.com/es-joy/jsdoccomment/blob/6aae5ea306015096e3d58cd22257e5222c54e3b4/src/jsdoccomment.js#L283
    //
    // OXC Semanticとして、どう振る舞うべきかは悩ましいが、現状の実装としては、
    // - TypeScriptライクな直感的なアタッチ
    // - 用途に応じて、単一のJSDocを取得するか、すべてのJSDocを取得するか選ぶ
    // というハイブリッドな形になっている。
    pub fn retrieve_attached_jsdoc(&mut self, kind: &AstKind<'a>) -> bool {
        // This is not enough compare to TypeScript's `canHaveJSDoc()`, should expand if needed
        if !(kind.is_statement()
            || kind.is_declaration()
            || matches!(kind, AstKind::ParenthesizedExpression(_)))
        {
            return false;
        }

        // 1. Retrieve every kind of leading comments for this node
        let span = kind.span();
        let mut leading_comments = vec![];
        for (start, comment) in self.trivias.comments().range(..span.start) {
            if !self.leading_comments_seen.contains(start) {
                leading_comments.push((start, comment));
            }
            self.leading_comments_seen.insert(*start);
        }

        // 2. Filter and parse JSDoc comments only
        let leading_jsdoc_comments = leading_comments
            .iter()
            .filter(|(_, comment)| comment.is_multi_line())
            .filter_map(|(start, comment)| {
                let comment_span = Span::new(**start, comment.end());
                // Inside of marker: /*_CONTENT_*/
                let comment_content = comment_span.source_text(self.source_text);
                // Should start with "*": /**_CONTENT_*/
                if !comment_content.starts_with('*') {
                    return None;
                }
                Some(comment_content)
            })
            .map(|comment_content| {
                // Shold remove the very first `*`?
                JSDocComment::new(comment_content)
            })
            .collect::<Vec<_>>();

        // 3. Save and return `true` to mark JSDoc flag
        if !leading_jsdoc_comments.is_empty() {
            self.docs.insert(span, leading_jsdoc_comments);
            return true;
        }

        false
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::{SourceType, Span};

    use crate::{jsdoc::JSDocComment, Semantic, SemanticBuilder};

    fn build_semantic<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: Option<SourceType>,
    ) -> Semantic<'a> {
        let source_type = source_type.unwrap_or_default();
        let ret = Parser::new(allocator, source_text, source_type).parse();
        let program = allocator.alloc(ret.program);
        let semantic = SemanticBuilder::new(source_text, source_type)
            .with_trivias(ret.trivias)
            .build(program)
            .semantic;
        semantic
    }

    #[allow(clippy::cast_possible_truncation)]
    fn get_jsdoc<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        symbol: &'a str,
        source_type: Option<SourceType>,
    ) -> Option<Vec<JSDocComment<'a>>> {
        let semantic = build_semantic(allocator, source_text, source_type);
        let start = source_text.find(symbol).unwrap_or(0) as u32;
        let span = Span::new(start, start + symbol.len() as u32);
        semantic.jsdoc().get_by_span(span)
    }

    fn test_jsdoc_found(source_text: &str, symbol: &str, source_type: Option<SourceType>) {
        let allocator = Allocator::default();
        assert!(
            get_jsdoc(&allocator, source_text, symbol, source_type).is_some(),
            "{symbol} not found in {source_text}"
        );
    }

    fn test_jsdoc_not_found(source_text: &str, symbol: &str) {
        let allocator = Allocator::default();
        assert!(
            get_jsdoc(&allocator, source_text, symbol, None).is_none(),
            "{symbol} found in {source_text}"
        );
    }

    #[test]
    fn not_found() {
        let source_texts = [
            ("function foo() {}", "function foo() {}"),
            ("// test", "function foo() {}"),
            ("function foo() {}", "function foo() {}"),
            ("/* test */function foo() {}", "function foo() {}"),
            ("/** test */ ; function foo() {}", "function foo() {}"),
            ("/** test */ function foo1() {} function foo() {}", "function foo() {}"),
            ("function foo() {} /** test */", "function foo() {}"),
        ];
        for (source_text, target) in source_texts {
            test_jsdoc_not_found(source_text, target);
        }
    }

    #[test]
    fn found() {
        let source_texts = [
            ("/** test */function foo() {}", "function foo() {}"),
            ("/*** test */function foo() {}", "function foo() {}"),
            (
                "
            /** test */
        function foo() {}",
                "function foo() {}",
            ),
            (
                "/** test */
                function foo() {}",
                "function foo() {}",
            ),
            (
                "/**
             * test
             * */
            function foo() {}",
                "function foo() {}",
            ),
            (
                "/** test */
            function foo() {}",
                "function foo() {}",
            ),
            (
                "/** test */
            // noop
            function foo() {}",
                "function foo() {}",
            ),
            (
                "/** test */
            /*noop*/
            function foo() {}",
                "function foo() {}",
            ),
            ("/** foo1 */ function foo1() {} /** test */ function foo() {}", "function foo() {}"),
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
    fn get_all_jsdoc() {
        let allocator = Allocator::default();
        let semantic = build_semantic(
            &allocator,
            r"
            /** 1. ; */
            ;
            /** 2. class X {} *//** 3. class X {} */
            class X {
                /** 4. foo */
                foo = /** 5. () */ (() => {});
            }

            /** Not attached! */
            ",
            Some(SourceType::default()),
        );
        assert_eq!(semantic.jsdoc().iter_all().count(), 6);
    }
}
