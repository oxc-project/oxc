use rustc_hash::{FxHashMap, FxHashSet};

use oxc_allocator::Allocator;
use oxc_ast::{AstKind, ast};
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::{AstNode, Semantic, SemanticBuilder};
use oxc_span::SourceType;

use crate::get_parse_options;

pub fn detect_code_removal(
    before_text: &str,
    after_text: &str,
    source_type: SourceType,
) -> Option<String> {
    let before_stats = collect(before_text, source_type);
    let after_stats = collect(after_text, source_type);

    diff(&before_stats, &after_stats)
}

/// Collect statistics from source code.
fn collect(code: &str, source_type: SourceType) -> StatsCollector {
    let allocator = Allocator::default();
    let parser = Parser::new(&allocator, code, source_type).with_options(get_parse_options());
    let ParserReturn { program, errors, .. } = parser.parse();

    let mut collector = StatsCollector::default();

    // If there are parse errors, skip further analysis.
    // This will be reported in `diff()` later.
    if !errors.is_empty() {
        collector.has_parse_error = true;
        return collector;
    }

    // Using semantic analysis here only to get parent node
    let semantic_ret = SemanticBuilder::new().build(&program);
    collector.collect(&program, &semantic_ret.semantic)
}

/// Check if there's a difference (= code removal) between before and after formatting.
fn diff(before: &StatsCollector, after: &StatsCollector) -> Option<String> {
    // Simply counts differences in node counts.
    // `debug_name()` which contains node type and its details is used as the key.
    fn diff_counts(before: &Counter, after: &Counter) -> Option<Vec<String>> {
        let mut errors = Vec::new();

        let mut all_key_names: FxHashSet<_> = before.keys().collect();
        all_key_names.extend(after.keys());

        for key in all_key_names {
            let before_count = before.get(key).copied().unwrap_or(0);
            let after_count = after.get(key).copied().unwrap_or(0);

            if before_count != after_count {
                errors
                    .push(format!("Count mismatch for '{key}': {before_count} -> {after_count}",));
            }
        }

        (!errors.is_empty()).then_some(errors)
    }

    // Compare block comments by their count.
    // We should check content but it is not straightforward,
    // leading whitespaces inside block comment also removed and `*` is aligned.
    fn diff_block_comments(before: &[String], after: &[String]) -> Option<String> {
        let before_count = before.len();
        let after_count = after.len();
        (before_count != after_count)
            .then_some(format!("BlockComment count mismatch: {before_count} -> {after_count}"))
    }

    // Compare line comments by their content.
    // Count is not enough since line comments can be merged.
    fn diff_line_comments(before: &[String], after: &[String]) -> Option<String> {
        // Happy path!
        if before == after {
            return None;
        }

        // Sometimes, formatter trims trailing whitespaces.
        // e.g.
        // ```
        // // if extra whitespaces here ->
        // ```
        // (rustfmt also removes...)
        // -> `if extra whitespaces here ->`
        //
        // Sometimes, line comments are merged.
        // e.g.
        // ```
        // for (x
        // in //a
        // y); //b
        // ```
        // -> `for (x in y); //a //b` = Comment: 2 -> 1
        for bf in before {
            if after.iter().any(|af| bf.trim_end() == af || af.contains(bf)) {
                continue;
            }
            return Some(format!("LineComment mismatch: `{bf}` not found"));
        }

        None
    }

    if before.has_parse_error {
        unreachable!("Do not expect to be called with code having parse error");
    }
    if after.has_parse_error {
        return Some("Parse error found after formatting".to_string());
    }

    let mut errors = Vec::new();
    errors.extend(diff_block_comments(&before.block_comments, &after.block_comments));
    errors.extend(diff_line_comments(&before.line_comments, &after.line_comments));
    errors.extend(diff_counts(&before.node_counts, &after.node_counts).unwrap_or_default());

    (!errors.is_empty()).then_some(errors.join("\n"))
}

type Counter = FxHashMap<String, usize>;

#[derive(Debug, Default)]
struct StatsCollector {
    has_parse_error: bool,
    block_comments: Vec<String>,
    line_comments: Vec<String>,
    node_counts: Counter,
}

impl StatsCollector {
    #[must_use]
    fn collect<'a>(mut self, program: &ast::Program<'a>, semantic: &Semantic<'a>) -> Self {
        for comment in &program.comments {
            // See `diff()` logic above for why content is needed
            let content = comment.span.source_text(program.source_text).to_string();
            if comment.is_block() {
                self.block_comments.push(content);
            } else {
                self.line_comments.push(content);
            }
        }

        for node in semantic.nodes().iter() {
            self.handle_node(node, semantic);
        }

        self
    }

    fn handle_node<'a>(&mut self, node: &AstNode<'a>, semantic: &Semantic<'a>) {
        let kind = node.kind();

        // `;` can be safely removed.
        // e.g. `;[];` -> `[];` = EmptyStatement: 1 -> 0
        if matches!(kind, AstKind::EmptyStatement(_)) {
            return;
        }

        // `TSUnionType` can have unnecessary leading `|`.
        // If there is such leading `|` with only one type, they can be removed.
        // e.g. `type T = | A` -> `type T = A` = TSUnionType: 1 -> 0
        if matches!(kind, AstKind::TSUnionType(u) if u.types.len() == 1) {
            return;
        }
        // `TSIntersectionType` can have unnecessary leading `&` too.
        // e.g. `type T = & A` -> `type T = A` = TSIntersectionType: 1 -> 0
        if matches!(kind, AstKind::TSIntersectionType(i) if i.types.len() == 1) {
            return;
        }

        // Useless parentheses can be safely removed.
        // e.g. `(a)` -> `a`
        // e.g. `type T = (A | B)` -> `type T = A | B`
        //
        // However, at the same time, some nodes are affected by parentheses removal.
        // e.g. `(a, (b, c))` -> `(a, b, c)` = SequenceExpression: 2 -> 1
        // e.g. `(a?.b)?.c` -> `a?.b?.c` = ChainExpression: 2 -> 1
        // e.g. `(a)()` -> `a()` = CallExpression(<computed>) -> CallExpression(a)
        //
        // Track of such nodes is not straightforward.
        // To detect whether only truly unnecessary parentheses were removed,
        // we need to consider semantics, scope, etc...
        //
        // NOTE: By this reason, at least for now, do not count parentheses and the affected nodes.
        // For the first place, there should be no parentheses by `preserve_parens: false`.
        if matches!(
            kind,
            AstKind::ParenthesizedExpression(_)
                | AstKind::TSParenthesizedType(_)
                | AstKind::ChainExpression(_)
                | AstKind::SequenceExpression(_)
        ) {
            return;
        }

        // Count by `debug_name` which contains the node type and its details.
        // If this is too strict, we can relax it later.
        let node_name = kind.debug_name().to_string();
        let parent_kind = semantic.nodes().parent_kind(node.id());

        // Object-like keys can be formatted differently based on quote options.
        // e.g. `{ "key": value }` -> `{ key: value }`
        // Therefore, we should count their value instead of their node type.
        if matches!(
            parent_kind,
            AstKind::ObjectProperty(_)
                | AstKind::MethodDefinition(_)
                | AstKind::PropertyDefinition(_)
                | AstKind::ImportAttribute(_)
                | AstKind::TSPropertySignature(_)
                | AstKind::TSLiteralType(_),
        ) && matches!(
            kind,
            AstKind::IdentifierName(_) | AstKind::StringLiteral(_) | AstKind::NumericLiteral(_)
        ) {
            for prefix in ["StringLiteral(", "IdentifierName(", "NumericLiteral("] {
                if let Some(rest) = node_name.strip_prefix(prefix)
                    && let Some(value) = rest.strip_suffix(')')
                {
                    *self.node_counts.entry(format!("OBJECT_LIKE_KEYS({value})")).or_insert(0) += 1;
                    return;
                }
            }
        }

        // `JSXText` with only whitespace can be safely removed.
        // e.g.
        // ```
        // return (
        //   <div>
        //     {children}
        //   </div>
        // )
        // ```
        // -> `return (<div>{children}</div>)` = JSXText(\n): 2 -> 0
        //
        // NOTE: In addition, there are many cases related to `JSXText` where whitespaces are changed.
        // Just skip counting `JSXText` with only whitespaces.
        if matches!(kind, AstKind::JSXText(t) if t.value.trim().is_empty()) {
            return;
        }

        // `JSXExpressionContainer` with a single whitespace `StringLiteral` will be removed.
        // e.g. `<div>{" "}</div>` -> `<div> </div>`
        // = JSXExpressionContainer: 1 -> 0, StringLiteral: 1 -> 0, JSXText(" "): 0 -> 1
        // e.g. `<div>{" "}{' '}</div>` -> `<div> </div>`
        // = JSXExpressionContainer: 2 -> 0, StringLiteral: 1 -> 0, JSXText(" "): 0 -> 1
        if matches!(kind, AstKind::JSXExpressionContainer(c)
            if matches!(&c.expression, ast::JSXExpression::StringLiteral(s) if s.value == " "))
        {
            // Skip `JSXExpressionContainer` containing only `StringLiteral(" ")`
            return;
        }
        if matches!(parent_kind, AstKind::JSXExpressionContainer(_))
            && matches!(kind, AstKind::StringLiteral(s) if s.value == " ")
        {
            // Skip `StringLiteral(" ")` inside `JSXExpressionContainer`
            return;
        }

        let count_key = match kind {
            // `JSXText` may contain redundant whitespaces.
            // e.g. `<p>World    </p>` -> `<p>World </p>`
            // Redundant whitespaces can be truncated even if they are inside.
            // e.g. `<p>abc   :  def</p>` -> `<p>abc : def</p>`
            AstKind::JSXText(t) if t.value != " " => {
                format!("JSX_TEXT({})", t.value.split_whitespace().collect::<Vec<_>>().join(" "))
            }
            // When ParenthesizedExpression is removed, CallExpression(calee name) may be affected.
            // e.g. `(a?.b)?.()` -> `a?.b?.()`
            // CallExpression(<computed>) -> CallExpression(b)
            AstKind::CallExpression(_) => "CALL_EXPRESSION".to_string(),
            _ => node_name,
        };
        *self.node_counts.entry(count_key).or_insert(0) += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_code_removal() {
        for (code1, code2) in [
            ("const a = 1;", "const b = 1;"),
            ("const a = 1; // variable a", "const a = 1;"),
            ("/* block comment */ const b = 2;", "const b = 2;"),
        ] {
            let source_type = SourceType::default().with_typescript(true).with_jsx(true);

            let stats1 = collect(code1, source_type);
            let stats2 = collect(code2, source_type);

            assert_ne!(diff(&stats1, &stats2), None, "Expected diff not found:\n{code1}");
        }
    }

    #[test]
    fn test_no_diff_for_same_code() {
        for (code1, code2) in [
            ("const a = 1;", "const a = 1;"),
            ("for ((let.a) of foo);", "for ((let).a of foo);"),
            ("for ((let[a]) of foo);", "for ((let)[a] of foo);"),
            ("for ((let.a) in foo);", "for (let.a in foo);"),
            ("<div>{' '}</div>", "<div> </div>"),
            ("type T = | A;", "type T = A;"),
            ("type T = & A;", "type T = A;"),
            ("(a);", "a;"),
            ("(a)();", "a();"),
            ("(a, (b, c));", "(a, b, c);"),
            ("(a?.b)?.c;", "a?.b?.c;"),
            ("(a.b)?.().c;", "a.b?.().c;"),
            ("(a?.b)?.().c", "a?.b?.().c"),
            ("for ((let)[a] in foo);", "for ((let)[a] in foo);"),
        ] {
            let source_type = SourceType::default().with_typescript(true).with_jsx(true);

            let stats1 = collect(code1, source_type);
            let stats2 = collect(code2, source_type);

            if let Some(diff) = diff(&stats1, &stats2) {
                panic!("Unexpected diff found:\n{diff}");
            }
        }
    }
}
