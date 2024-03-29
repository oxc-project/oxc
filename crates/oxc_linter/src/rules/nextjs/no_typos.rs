use oxc_ast::{
    ast::{BindingPatternKind, Declaration, ModuleDeclaration},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::phf_set;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-next(no-typos): {0} may be a typo. Did you mean {1}?")]
#[diagnostic(severity(warning), help("Prevent common typos in Next.js's data fetching functions"))]
struct NoTyposDiagnostic(String, String, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoTypos;

declare_oxc_lint!(
    /// ### What it does
    /// Prevent common typos in Next.js's data fetching functions
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// export default function Page() {
    ///   return <div></div>;
    /// }
    /// export async function getServurSideProps(){};
    /// ```
    NoTypos,
    correctness
);

const NEXTJS_DATA_FETCHING_FUNCTIONS: phf::Set<&'static str> = phf_set! {
    "getStaticProps",
    "getStaticPaths",
    "getServerSideProps",
};

// 0 is the exact match
const THRESHOLD: i32 = 1;

impl Rule for NoTypos {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let Some(path) = ctx.file_path().to_str() else { return };
        let Some(path_after_pages) = path.split("pages").nth(1) else { return };
        if path_after_pages.starts_with("/api") {
            return;
        }
        if let AstKind::ModuleDeclaration(ModuleDeclaration::ExportNamedDeclaration(en_decl)) =
            node.kind()
        {
            if let Some(ref decl) = en_decl.declaration {
                match decl {
                    Declaration::VariableDeclaration(decl) => {
                        for decl in &decl.declarations {
                            let BindingPatternKind::BindingIdentifier(id) = &decl.id.kind else {
                                continue;
                            };
                            let Some(potential_typo) = get_potential_typo(&id.name) else {
                                continue;
                            };
                            ctx.diagnostic(NoTyposDiagnostic(
                                id.name.to_string(),
                                potential_typo.to_string(),
                                id.span,
                            ));
                        }
                    }
                    Declaration::FunctionDeclaration(decl) => {
                        let Some(id) = &decl.id else { return };
                        let Some(potential_typo) = get_potential_typo(&id.name) else { return };
                        ctx.diagnostic(NoTyposDiagnostic(
                            id.name.to_string(),
                            potential_typo.to_string(),
                            id.span,
                        ));
                    }
                    _ => {}
                }
            }
        }
    }
}

fn get_potential_typo(fn_name: &str) -> Option<&str> {
    let mut potential_typos: Vec<_> = NEXTJS_DATA_FETCHING_FUNCTIONS
        .iter()
        .map(|&o| {
            let distance = min_distance(o, fn_name);
            (o, distance)
        })
        .filter(|&(_, distance)| distance <= THRESHOLD as usize && distance > 0)
        .collect();

    potential_typos.sort_by(|a, b| a.1.cmp(&b.1));

    potential_typos.first().map(|(option, _)| *option)
}

// the minimum number of operations required to convert string a to string b.
fn min_distance(a: &str, b: &str) -> usize {
    let m = a.len();
    let n = b.len();

    if m < n {
        return min_distance(b, a);
    }

    if n == 0 {
        return m;
    }

    let mut previous_row: Vec<usize> = (0..=n).collect();

    for (i, s1) in a.chars().enumerate() {
        let mut current_row = vec![i + 1];
        for (j, s2) in b.chars().enumerate() {
            let insertions = previous_row[j + 1] + 1;
            let deletions = current_row[j] + 1;
            let substitutions = previous_row[j] + usize::from(s1 != s2);
            current_row.push(insertions.min(deletions).min(substitutions));
        }
        previous_row = current_row;
    }
    previous_row[n]
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

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
            r"
                export default function Page() {
                return <div></div>;
                }
                export const getStaticpaths = async () => {};
                export const getStaticProps = async () => {};
            ",
            None,
            None,
            Some(PathBuf::from("pages/api/test.tsx")),
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

    Tester::new(NoTypos::NAME, pass, fail).test_and_snapshot();
}
