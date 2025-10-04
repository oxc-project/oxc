use oxc_diagnostics::OxcDiagnostic;
use oxc_index::{IndexVec, define_nonmax_u32_index_type};
use oxc_macros::declare_oxc_lint;
use oxc_regular_expression::{
    ast::LookAroundAssertionKind,
    visit::{RegExpAstKind, Visit},
};
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::run_on_regex_node};

fn no_useless_backreference_diagnostic(
    span: Span,
    problem: &Problem,
    back_reference: &str,
    group: &str,
) -> OxcDiagnostic {
    match problem {
        Problem::Nested =>OxcDiagnostic::warn(format!("Backreference '{back_reference}' will be ignored. It references group '{group}' from within that group.")).with_label(span),
        Problem::Disjunctive => OxcDiagnostic::warn(format!("Backreference '{back_reference}' will be ignored. It references group '{group}' which is in another alternative.")).with_label(span),
        Problem::Forward => OxcDiagnostic::warn(format!("Backreference '{back_reference}' will be ignored. It references group '{group}' which appears later in the pattern.")).with_label(span),
        Problem::Backward => OxcDiagnostic::warn(format!("Backreference '{back_reference}' will be ignored. It references group '{group}' which appears before in the same lookbehind.")).with_label(span),
        Problem::IntoNegativeLookaround => OxcDiagnostic::warn(format!("Backreference '{back_reference}' will be ignored. It references group '{group}' which is in a negative lookaround.")).with_label(span),
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessBackreference;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows backreferences in regular expressions that will always be ignored
    /// because the capture group they refer to has not matched and cannot match
    /// at the time the backreference is evaluated.
    ///
    /// ### Why is this bad?
    ///
    /// Useless backreferences can lead to confusing or misleading regular expressions.
    /// They may give the impression that a group’s value is being reused, but due to
    /// the structure of the pattern (e.g., order of evaluation, disjunctions, or negative
    /// lookarounds), the group has not matched anything — so the reference always
    /// resolves to an empty string. This is almost always a mistake and makes patterns
    /// harder to understand and maintain.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /\1(a)/;                     // backreference appears before group
    /// /(a|\1b)/;                   // group and reference are in different alternatives
    /// /(?<=\1(a))b/;               // backreference used before group in lookbehind
    /// /\1(?!(a))/;                 // group is inside negative lookahead
    /// /(a\1)/;                     // backreference is inside its own group
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /(a)\1/;                     // valid — backreference follows completed group
    /// /(?<name>a)\k<name>/;        // named group used properly
    /// /(?:a|(b))\1/;               // backreference only used when group matches
    /// ```
    NoUselessBackreference,
    eslint,
    correctness
);

impl Rule for NoUselessBackreference {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        run_on_regex_node(node, ctx, |pattern, _span| {
            let mut collector = RegexCollector::new();
            collector.visit_pattern(pattern);

            for bref in &collector.backrefs {
                if let Some((problem, cap_group_span, bref_group_span)) =
                    problems_for_backref(bref, &collector.nodes, &collector.groups)
                {
                    ctx.diagnostic(no_useless_backreference_diagnostic(
                        bref.span,
                        &problem,
                        ctx.source_range(bref_group_span),
                        ctx.source_range(cap_group_span),
                    ));
                }
            }
        });
    }
}

enum Problem {
    Nested,
    Disjunctive,
    Forward,
    Backward,
    IntoNegativeLookaround,
}

#[derive(Debug, Clone)]
struct GroupInfo<'a> {
    name: Option<&'a str>,
    span: Span,
    path: Vec<RegexNodeId>,
}

#[derive(Debug, Clone)]
struct BackRefInfo<'a> {
    target: BackRefInfoTarget<'a>,
    span: Span,
    path: Vec<RegexNodeId>,
}

#[derive(Debug, Clone)]
enum BackRefInfoTarget<'a> {
    Index(u32),
    Name(&'a str),
}

define_nonmax_u32_index_type! {
    pub struct RegexNodeId;
}

#[derive(Debug)]
struct RegexCollector<'a> {
    nodes: IndexVec<RegexNodeId, RegExpAstKind<'a>>,
    stack: Vec<RegexNodeId>,

    groups: Vec<GroupInfo<'a>>,
    backrefs: Vec<BackRefInfo<'a>>,
}

impl RegexCollector<'_> {
    fn new() -> Self {
        Self { nodes: IndexVec::new(), stack: Vec::new(), groups: Vec::new(), backrefs: Vec::new() }
    }

    #[inline]
    fn pop(&mut self) {
        self.stack.pop();
    }

    fn current_path(&self) -> Vec<RegexNodeId> {
        self.stack.clone()
    }
}

impl<'ast> Visit<'ast> for RegexCollector<'ast> {
    fn enter_node(&mut self, kind: RegExpAstKind<'ast>) {
        let id = self.nodes.push(kind);
        self.stack.push(id);

        match kind {
            RegExpAstKind::CapturingGroup(group) => {
                self.groups.push(GroupInfo {
                    name: group.name.map(|n| n.as_str()),
                    span: group.span,
                    path: self.current_path(),
                });
            }
            RegExpAstKind::IndexedReference(idx_ref) => {
                self.backrefs.push(BackRefInfo {
                    target: BackRefInfoTarget::Index(idx_ref.index),
                    span: idx_ref.span,
                    path: self.current_path(),
                });
            }
            RegExpAstKind::NamedReference(named) => {
                self.backrefs.push(BackRefInfo {
                    target: BackRefInfoTarget::Name(named.name.as_str()),
                    span: named.span,
                    path: self.current_path(),
                });
            }
            _ => {}
        }
    }

    fn leave_node(&mut self, _kind: RegExpAstKind<'ast>) {
        self.pop();
    }
}

/// Determines if a backreference is useless — meaning it references a capturing
/// group that hasn't (and cannot) participate at the time it's evaluated.
///
/// Returns the reason as a `Problem` if one is found; otherwise returns `None`.
///
/// # Problem types and examples:
///
/// - `Problem::Nested`:
///   `/ (a\1) /`
///   ↳ Backref appears *inside* the group it references — it can't match because the group is still open.
///
/// - `Problem::Disjunctive`:
///   `/ ((a)|\1b) /`
///   ↳ Backref and group are in different alternatives — only one branch runs, so the group can't be matched.
///
/// - `Problem::Backward`:
///   `/ \1(a) /`
///   ↳ Backref appears before the group. Left-to-right matching means the group hasn’t matched yet.
///
/// - `Problem::Forward`:
///   `/ (?<=\1(a)) /`
///   ↳ In a lookbehind (right-to-left match), the group appears *after* the reference — invalid.
///
/// - `Problem::IntoNegativeLookaround`:
///   `/ \1(?!(a)) /`
///   ↳ Group is inside a negative lookaround — by definition, it can never successfully match.
fn problems_for_backref<'a>(
    bref: &'a BackRefInfo<'a>,
    nodes: &IndexVec<RegexNodeId, RegExpAstKind<'a>>,
    groups: &'a [GroupInfo<'a>],
) -> Option<(
    Problem,
    /* span of the backreference */ Span,
    /* span of the capture group */ Span,
)> {
    let cap_group = match bref.target {
        BackRefInfoTarget::Index(i) => groups.get(i as usize - 1),
        BackRefInfoTarget::Name(cap_group_name) => {
            groups.iter().find(|g| g.name.is_some_and(|n| n == cap_group_name))
        }
    };
    let Some(cap_group) = cap_group else {
        debug_assert!(
            false,
            "a backreference must have a corresponding capture group, else it is an octal escape"
        );
        return None;
    };

    // Problem::Nested
    // In this scenario, the backreference appears *inside* the group it refers to.
    // e.g. `/(a\1)/`
    //        ^^^^^ capture group
    //          ^^ backreference
    // This is invalid because when the regex engine reaches the backreference,
    // the group it points to has not finished matching yet — it’s still open.
    // As a result, the group has not captured any text at that point, so the
    // backreference is always interpreted as the empty string.
    // While this does not cause a syntax error, the reference serves no purpose
    // and is almost certainly a logic mistake or misunderstanding of how regex works.
    if cap_group.span.contains_inclusive(bref.span) {
        return Some((Problem::Nested, cap_group.span, bref.span));
    }

    let index_of_lowest_common_ancestor = {
        let mut i = 0;
        let mut j = 0;

        while i + 1 < bref.path.len()
            && j + 1 < cap_group.path.len()
            && bref.path[i + 1] == cap_group.path[j + 1]
        {
            i += 1;
            j += 1;
        }
        j + 1
    };

    let group_cut = &cap_group.path[index_of_lowest_common_ancestor..];
    let common_path = &bref.path[..index_of_lowest_common_ancestor];

    let is_matching_backwards = common_path
        .iter()
        .rev()
        .map(|node_id| nodes[*node_id])
        .find_map(|node| match node {
            RegExpAstKind::LookAroundAssertion(l) => Some(l),
            _ => None,
        })
        .is_some_and(|v| {
            matches!(
                v.kind,
                LookAroundAssertionKind::Lookbehind | LookAroundAssertionKind::NegativeLookbehind
            )
        });

    // Problem::Disjunctive
    // The group and backreference reside in *different alternatives* of the same disjunction.
    // e.g. `/((a)|\1b)/`
    //         ^        capturing group in 1st branch
    //            ^^^   backreference in 2nd branch
    // Only one branch of a disjunction (`|`) is executed at runtime. So if the group is matched,
    // the backreference is never reached; and if the backreference is evaluated, the group didn't match.
    // Therefore, the backreference is always empty — it's unreachable in practice.
    if group_cut.first().is_some_and(|id| matches!(nodes[*id], RegExpAstKind::Alternative(_))) {
        return Some((Problem::Disjunctive, cap_group.span, bref.span));
    }

    // Problem::Backward
    // A "forward" reference during left-to-right matching (normal regex mode).
    // e.g. `/\1(a)/`
    //      ^^ backreference
    //         ^^^ capture group
    // Here, the backreference appears *before* the group has started.
    // Since regex engines evaluate patterns from left to right,
    // the capture group has not matched anything by the time `\1` is evaluated.
    // So the backreference resolves to the empty string and is effectively useless.
    if !is_matching_backwards && bref.span.end <= cap_group.span.start {
        return Some((Problem::Backward, cap_group.span, bref.span));
    }

    // Problem::Forward
    // A forward reference inside a lookbehind (right-to-left matching).
    // e.g. `/(?<=\1(a))/`
    //            ^^ backreference
    //              ^^^ capture group
    // Lookbehinds are evaluated from right to left.
    // In this direction, the group appears *after* the backreference, so it has not matched yet
    // when `\1` is evaluated. As a result, the backreference resolves to the empty string.
    if is_matching_backwards && cap_group.span.end <= bref.span.start {
        return Some((Problem::Forward, cap_group.span, bref.span));
    }

    // Problem::IntoNegativeLookaround
    // The capture group resides inside a negative lookahead or lookbehind assertion.
    // e.g. `/\1(?!(a))/`
    //        ^^ backreference
    //          ^^^^^^^ negative lookahead containing capture group
    // In negative lookarounds, the contents must *fail* to match in order for the overall
    // assertion to pass. This means the group inside can never match successfully.
    // As a result, the backreference points to a group that will never have a value,
    // and is always empty.
    if group_cut.iter().map(|node_id| nodes[*node_id]).any(|node| match node {
        RegExpAstKind::LookAroundAssertion(l) => matches!(
            l.kind,
            LookAroundAssertionKind::NegativeLookahead
                | LookAroundAssertionKind::NegativeLookbehind
        ),
        _ => false,
    }) {
        return Some((Problem::IntoNegativeLookaround, cap_group.span, bref.span));
    }

    None
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"'\1(a)'",
        r"regExp('\\1(a)')",
        r"new Regexp('\\1(a)', 'u')",
        r"RegExp.foo('\\1(a)', 'u')",
        r"new foo.RegExp('\\1(a)')",
        r"RegExp(p)",
        r"new RegExp(p, 'u')",
        r"RegExp('\\1(a)' + suffix)",
        r"new RegExp(`${prefix}\\1(a)`)",
        r"let RegExp; new RegExp('\\1(a)');",
        r"function foo() { var RegExp; RegExp('\\1(a)', 'u'); }",
        r"function foo() { var RegExp; RegExp('\\1(a)', `u`); }",
        r"function foo(RegExp) { new RegExp('\\1(a)'); }",
        r"if (foo) { const RegExp = bar; RegExp('\\1(a)'); }",
        // we don't support globals off yet
        //r#"/* globals RegExp:off */ new RegExp('\\1(a)');"#,
        //r#"RegExp('\\1(a)');"#, // {				"globals": { "RegExp": "off" },			},
        r"/(?:)/",
        r"/(?:a)/",
        r"new RegExp('')",
        r"RegExp('(?:a)|(?:b)*')",
        r"/^ab|[cd].\n$/",
        r"/(a)/",
        r"RegExp('(a)|(b)')",
        r"new RegExp('\\n\\d(a)')",
        r"/\0(a)/",
        r"/\0(a)/u",
        r"/(?<=(a))(b)(?=(c))/",
        r"/(?<!(a))(b)(?!(c))/",
        r"/(?<foo>a)/",
        r"RegExp('\1(a)')",
        r"RegExp('\\\\1(a)')",
        r"/\\1(a)/",
        r"/\1/",
        r"/^\1$/",
        r"/\2(a)/",
        r"/\1(?:a)/",
        r"/\1(?=a)/",
        r"/\1(?!a)/",
        r"/^[\1](a)$/",
        r"new RegExp('[\\1](a)')",
        r"/\11(a)/",
        r"/\k<foo>(a)/",
        r"/^(a)\1\\2$/",
        r"/(a)\1/",
        r"/(a).\1/",
        r"RegExp('(a)\\1(b)')",
        r"/(a)(b)\2(c)/",
        r"/(?<foo>a)\k<foo>/",
        r"new RegExp('(.)\\1')",
        r"RegExp('(a)\\1(?:b)')",
        r"/(a)b\1/",
        r"/((a)\2)/",
        r"/((a)b\2c)/",
        r"/^(?:(a)\1)$/",
        r"/^((a)\2)$/",
        r"/^(((a)\3))|b$/",
        r"/a(?<foo>(.)b\2)/",
        r"/(a)?(b)*(\1)(c)/",
        r"/(a)?(b)*(\2)(c)/",
        r"/(?<=(a))b\1/",
        r"/(?<=(?=(a)\1))b/",
        r"/(?<!\1(a))b/",
        r"/(?<=\1(a))b/",
        r"/(?<!\1.(a))b/",
        r"/(?<=\1.(a))b/",
        r"/(?<=(?:\1.(a)))b/",
        r"/(?<!(?:\1)((a)))b/",
        r"/(?<!(?:\2)((a)))b/",
        r"/(?=(?<=\1(a)))b/",
        r"/(?=(?<!\1(a)))b/",
        r"/(.)(?<=\2(a))b/",
        r"/^(a)\1|b/",
        r"/^a|(b)\1/",
        r"/^a|(b|c)\1/",
        r"/^(a)|(b)\2/",
        r"/^(?:(a)|(b)\2)$/",
        r"/^a|(?:.|(b)\1)/",
        r"/^a|(?:.|(b).(\1))/",
        r"/^a|(?:.|(?:(b)).(\1))/",
        r"/^a|(?:.|(?:(b)|c).(\1))/",
        r"/^a|(?:.|(?:(b)).(\1|c))/",
        r"/^a|(?:.|(?:(b)|c).(\1|d))/",
        r"/.(?=(b))\1/",
        r"/.(?<=(b))\1/",
        r"/a(?!(b)\1)./",
        r"/a(?<!\1(b))./",
        r"/a(?!(b)(\1))./",
        r"/a(?!(?:(b)\1))./",
        r"/a(?!(?:(b))\1)./",
        r"/a(?<!(?:\1)(b))./",
        r"/a(?<!(?:(?:\1)(b)))./",
        r"/(?<!(a))(b)(?!(c))\2/",
        r"/a(?!(b|c)\1)./",
        r"RegExp('\\1(a)[')",
        r"new RegExp('\\1(a){', 'u')",
        r"new RegExp('\\1(a)\\2', 'ug')",
        //r#"const flags = 'gus'; RegExp('\\1(a){', flags);"#,
        r"RegExp('\\1(a)\\k<foo>', 'u')",
        r"new RegExp('\\k<foo>(?<foo>a)\\k<bar>')",
        r"new RegExp('([[A--B]])\\1', 'v')",
        r"new RegExp('[[]\\1](a)', 'v')",
        r"/((?<foo>bar)\k<foo>|(?<foo>baz))/",
    ];

    let fail = vec![
        r"/(b)(\2a)/",
        r"/\k<foo>(?<foo>bar)/",
        r"RegExp('(a|bc)|\\1')",
        r"new RegExp('(?!(?<foo>\\n))\\1')",
        r"/(?<!(a)\1)b/",
        r"new RegExp('(\\1)')",
        r"/^(a\1)$/",
        r"/^((a)\1)$/",
        r"new RegExp('^(a\\1b)$')",
        r"RegExp('^((\\1))$')",
        r"/((\2))/",
        r"/a(?<foo>(.)b\1)/",
        r"/a(?<foo>\k<foo>)b/",
        r"/^(\1)*$/",
        r"/^(?:a)(?:((?:\1)))*$/",
        r"/(?!(\1))/",
        r"/a|(b\1c)/",
        r"/(a|(\1))/",
        r"/(a|(\2))/",
        r"/(?:a|(\1))/",
        r"/(a)?(b)*(\3)/",
        r"/(?<=(a\1))b/",
        r"/\1(a)/",
        r"/\1.(a)/",
        r"/(?:\1)(?:(a))/",
        r"/(?:\1)(?:((a)))/",
        r"/(?:\2)(?:((a)))/",
        r"/(?:\1)(?:((?:a)))/",
        r"/(\2)(a)/",
        r"RegExp('(a)\\2(b)')",
        r"/(?:a)(b)\2(c)/",
        r"/\k<foo>(?<foo>a)/",
        r"/(?:a(b)\2)(c)/",
        r"new RegExp('(a)(b)\\3(c)')",
        r"/\1(?<=(a))./",
        r"/\1(?<!(a))./",
        r"/(?<=\1)(?<=(a))/",
        r"/(?<!\1)(?<!(a))/",
        r"/(?=\1(a))./",
        r"/(?!\1(a))./",
        r"/(?<=(a)\1)b/",
        r"/(?<!.(a).\1.)b/",
        r"/(.)(?<!(b|c)\2)d/",
        r"/(?<=(?:(a)\1))b/",
        r"/(?<=(?:(a))\1)b/",
        r"/(?<=(a)(?:\1))b/",
        r"/(?<!(?:(a))(?:\1))b/",
        r"/(?<!(?:(a))(?:\1)|.)b/",
        r"/.(?!(?<!(a)\1))./",
        r"/.(?=(?<!(a)\1))./",
        r"/.(?!(?<=(a)\1))./",
        r"/.(?=(?<=(a)\1))./",
        r"/(a)|\1b/",
        r"/^(?:(a)|\1b)$/",
        r"/^(?:(a)|b(?:c|\1))$/",
        r"/^(?:a|b(?:(c)|\1))$/",
        r"/^(?:(a(?!b))|\1b)+$/",
        r"/^(?:(?:(a)(?!b))|\1b)+$/",
        r"/^(?:(a(?=a))|\1b)+$/",
        r"/^(?:(a)(?=a)|\1b)+$/",
        r"/.(?:a|(b)).|(?:(\1)|c)./",
        r"/.(?!(a)|\1)./",
        r"/.(?<=\1|(a))./",
        r"/a(?!(b)).\1/",
        r"/(?<!(a))b\1/",
        r"/(?<!(a))(?:\1)/",
        r"/.(?<!a|(b)).\1/",
        r"/.(?!(a)).(?!\1)./",
        r"/.(?<!(a)).(?<!\1)./",
        r"/.(?=(?!(a))\1)./",
        r"/.(?<!\1(?!(a)))/",
        r"/\1(a)(b)\2/",
        r"/\1(a)\1/",
        r"/\1(a)\2(b)/",
        r"/\1.(?<=(a)\1)/",
        r"/(?!\1(a)).\1/",
        r"/(a)\2(b)/; RegExp('(\\1)');",
        r"RegExp('\\1(a){', flags);",
        // TODO: we don't support global references yet.
        //r#"const r = RegExp, p = '\\1', s = '(a)'; new r(p + s);"#,
        r"new RegExp('\\1([[A--B]])', 'v')",
        r"/\k<foo>((?<foo>bar)|(?<foo>baz))/",
        r"/((?<foo>bar)|\k<foo>(?<foo>baz))/",
        r"/\k<foo>((?<foo>bar)|(?<foo>baz)|(?<foo>qux))/",
        r"/((?<foo>bar)|\k<foo>(?<foo>baz)|(?<foo>qux))/",
        r"/((?<foo>bar)|\k<foo>|(?<foo>baz))/",
        r"/((?<foo>bar)|\k<foo>|(?<foo>baz)|(?<foo>qux))/",
        r"/((?<foo>bar)|(?<foo>baz\k<foo>)|(?<foo>qux\k<foo>))/",
        r"/(?<=((?<foo>bar)|(?<foo>baz))\k<foo>)/",
        r"/((?!(?<foo>bar))|(?!(?<foo>baz)))\k<foo>/",
    ];

    Tester::new(NoUselessBackreference::NAME, NoUselessBackreference::PLUGIN, pass, fail)
        .test_and_snapshot();
}
