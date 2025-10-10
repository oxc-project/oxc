use oxc_ast::{
    AstKind,
    ast::{BindingPatternKind, Declaration},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn no_typos_diagnostic(typo: &str, suggestion: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{typo}` may be a typo. Did you mean `{suggestion}`?"))
        .with_help(format!("Change `{typo}` to `{suggestion}`"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoTypos;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects common typos in Next.js data fetching function names.
    ///
    /// ### Why is this bad?
    ///
    /// Next.js will not call incorrectly named data fetching functions, causing pages to render without expected data.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// export default function Page() {
    ///   return <div></div>;
    /// }
    /// export async function getServurSideProps(){};
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// export default function Page() {
    ///   return <div></div>;
    /// }
    /// export async function getServerSideProps(){};
    /// ```
    NoTypos,
    nextjs,
    correctness,
    pending
);

const NEXTJS_DATA_FETCHING_FUNCTIONS: [&str; 3] =
    ["getStaticProps", "getStaticPaths", "getServerSideProps"];

// 0 is the exact match
const THRESHOLD: usize = 1;

impl Rule for NoTypos {
    fn should_run(&self, ctx: &ContextHost) -> bool {
        let path = ctx.file_path();
        let mut found_pages = false;
        for component in path.components() {
            if found_pages {
                return component.as_os_str() != "api";
            }
            if component.as_os_str() == "pages" {
                found_pages = true;
            }
        }
        false
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ExportNamedDeclaration(en_decl) = node.kind()
            && let Some(decl) = &en_decl.declaration
        {
            match decl {
                Declaration::VariableDeclaration(decl) => {
                    for decl in &decl.declarations {
                        if let BindingPatternKind::BindingIdentifier(id) = &decl.id.kind {
                            check_function_name(&id.name, id.span, ctx);
                        }
                    }
                }
                Declaration::FunctionDeclaration(decl) => {
                    if let Some(id) = &decl.id {
                        check_function_name(&id.name, id.span, ctx);
                    }
                }
                _ => {}
            }
        }
    }
}

fn check_function_name(name: &str, span: Span, ctx: &LintContext) {
    let mut potential_typos = NEXTJS_DATA_FETCHING_FUNCTIONS
        .into_iter()
        .filter_map(|o| {
            let distance = min_distance(o, name);
            (distance <= THRESHOLD && distance > 0).then_some((o, distance))
        })
        .collect::<Vec<_>>();

    potential_typos.sort_by(|a, b| a.1.cmp(&b.1));
    if let Some(suggestion) = potential_typos.first().map(|(option, _)| option) {
        ctx.diagnostic(no_typos_diagnostic(name, suggestion, span));
    }
}

fn min_distance(a: &str, b: &str) -> usize {
    if a.len() < b.len() {
        return min_distance(b, a);
    }

    let b_chars: Vec<char> = b.chars().collect();

    let n = b_chars.len();
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr: Vec<usize> = Vec::with_capacity(n + 1);
    for (i, ca) in a.chars().enumerate() {
        curr.clear();
        curr.push(i + 1);
        for (j, &cb) in b_chars.iter().enumerate() {
            curr.push((prev[j] + usize::from(ca != cb)).min(prev[j + 1] + 1).min(curr[j] + 1));
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        (
            r"
                export default function Page() {
                return <div></div>;
                }
                export const getStaticPaths = async () => {};
                export const getStaticProps = async () => {};
            ",
            None,
            None,
            Some(PathBuf::from("pages/test.tsx")),
        ),
        (
            r"
                export default function Page() {
                return <div></div>;
                }
                export const getServerSideProps = async () => {};
            ",
            None,
            None,
            Some(PathBuf::from("pages/test.tsx")),
        ),
        (
            r"
                export default function Page() {
                return <div></div>;
                }
                export async function getStaticPaths() {};
                export async function getStaticProps() {};
           	",
            None,
            None,
            Some(PathBuf::from("pages/test.tsx")),
        ),
        (
            r"
                export default function Page() {
                return <div></div>;
                }
                export async function getServerSideProps() {};
        	",
            None,
            None,
            Some(PathBuf::from("pages/test.tsx")),
        ),
        (
            r"
                export default function Page() {
                return <div></div>;
                }
                export async function getServerSidePropsss() {};
            ",
            None,
            None,
            Some(PathBuf::from("pages/test.tsx")),
        ),
        (
            r"
                export default function Page() {
                return <div></div>;
                }
                export async function getstatisPath() {};
            ",
            None,
            None,
            Some(PathBuf::from("pages/test.tsx")),
        ),
        // even though there is a typo match, this should not fail because a file is not inside pages directory
        (
            r"
                export default function Page() {
                return <div></div>;
                }
                export const getStaticpaths = async () => {};
                export const getStaticProps = async () => {};
            ",
            None,
            None,
            Some(PathBuf::from("test.tsx")),
        ),
        // even though there is a typo match, this should not fail because a file is inside pages/api directory
        (
            r"export const getStaticpaths = async () => {};",
            None,
            None,
            Some(PathBuf::from("pages/api/test.tsx")),
        ),
        (
            r"export const getStaticpaths = async () => {};",
            None,
            None,
            Some(PathBuf::from("pages\\api\\test.tsx")),
        ),
    ];

    let fail = vec![
        (
            r"
                export default function Page() {
                return <div></div>;
                }
                export const getStaticpaths = async () => {};
                export const getStaticProps = async () => {};
            ",
            None,
            None,
            Some(PathBuf::from("pages/test.tsx")),
        ),
        (
            r"
                export default function Page() {
                    return <div></div>;
                }
                export async function getStaticPathss(){};
                export async function getStaticPropss(){};
            ",
            None,
            None,
            Some(PathBuf::from("pages/test.tsx")),
        ),
        (
            r"
                export default function Page() {
                    return <div></div>;
                }
                export async function getServurSideProps(){};
            ",
            None,
            None,
            Some(PathBuf::from("pages/test.tsx")),
        ),
        (
            r"
                export default function Page() {
                    return <div></div>;
                }
                export const getServurSideProps = () => {};
            ",
            None,
            None,
            Some(PathBuf::from("pages/test.tsx")),
        ),
    ];

    Tester::new(NoTypos::NAME, NoTypos::PLUGIN, pass, fail).test_and_snapshot();
}
