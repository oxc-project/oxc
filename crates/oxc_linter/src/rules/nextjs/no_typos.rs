use oxc_ast::{
    AstKind,
    ast::{BindingPattern, Declaration},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::best_match,
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
                        if let BindingPattern::BindingIdentifier(id) = &decl.id {
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
    if let Some(suggestion) = best_match(name, NEXTJS_DATA_FETCHING_FUNCTIONS, THRESHOLD) {
        ctx.diagnostic(no_typos_diagnostic(name, suggestion, span));
    }
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
                export async function getStaticProps() {};",
            None,
            None,
            Some(PathBuf::from("pages/test.tsx")),
        ),
        (
            r"
                export default function Page() {
                return <div></div>;
                }
                export async function getServerSideProps() {};",
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
