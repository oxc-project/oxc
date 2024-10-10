use oxc_ast::{
    ast::{BindingPatternKind, Declaration, ModuleDeclaration},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::phf_set;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    AstNode,
};

fn no_typos_diagnostic(typo: &str, suggestion: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("{typo} may be a typo. Did you mean {suggestion}?"))
        .with_help("Prevent common typos in Next.js's data fetching functions")
        .with_label(span)
}

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
    correctness,
    pending
);

const NEXTJS_DATA_FETCHING_FUNCTIONS: phf::Set<&'static str> = phf_set! {
    "getStaticProps",
    "getStaticPaths",
    "getServerSideProps",
};

// 0 is the exact match
const THRESHOLD: i32 = 1;

impl Rule for NoTypos {
    fn should_run(&self, ctx: &ContextHost) -> bool {
        let Some(path) = ctx.file_path().to_str() else {
            return false;
        };
        let Some(path_after_pages) = path.split("pages").nth(1) else {
            return false;
        };
        if path_after_pages.starts_with("/api") {
            return false;
        }
        true
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
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
                            ctx.diagnostic(no_typos_diagnostic(
                                id.name.as_str(),
                                potential_typo,
                                id.span,
                            ));
                        }
                    }
                    Declaration::FunctionDeclaration(decl) => {
                        let Some(id) = &decl.id else { return };
                        let Some(potential_typo) = get_potential_typo(&id.name) else {
                            return;
                        };
                        ctx.diagnostic(no_typos_diagnostic(
                            id.name.as_str(),
                            potential_typo,
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

    for (i, s1) in a.char_indices() {
        let mut current_row = vec![i + 1];
        for (j, s2) in b.char_indices() {
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
