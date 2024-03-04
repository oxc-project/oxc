use std::{
    io,
    path::{Path, PathBuf},
};

use oxc_span::SourceType;
use serde::Deserialize;

use crate::{
    project_root,
    suite::{Case, Suite, TestResult},
};

const FIXTURES_PATH: &str = "tasks/coverage/test262/test";

#[derive(Debug, Clone, Deserialize, Default)]
pub struct MetaData {
    pub description: Box<str>,
    pub esid: Option<Box<str>>,
    pub es5id: Option<Box<str>>,
    pub es6id: Option<Box<str>>,
    #[serde(default)]
    pub info: Box<str>,
    #[serde(default)]
    pub features: Box<[Box<str>]>,
    #[serde(default)]
    pub includes: Box<[Box<str>]>,
    #[serde(default)]
    pub flags: Box<[TestFlag]>,
    #[serde(default)]
    pub negative: Option<Negative>,
    #[serde(default)]
    pub locale: Box<[Box<str>]>,
}

/// Individual test flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TestFlag {
    OnlyStrict,
    NoStrict,
    Module,
    Raw,
    Async,
    Generated,
    #[serde(rename = "CanBlockIsFalse")]
    CanBlockIsFalse,
    #[serde(rename = "CanBlockIsTrue")]
    CanBlockIsTrue,
    #[serde(rename = "non-deterministic")]
    NonDeterministic,
}

/// Negative test information structure.
#[derive(Debug, Clone, Deserialize)]
pub struct Negative {
    pub phase: Phase,
    #[serde(rename = "type")]
    pub error_type: Box<str>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Parse,
    Early,
    Resolution,
    Runtime,
}

impl Phase {
    pub fn is_runtime(self) -> bool {
        matches!(self, Self::Runtime)
    }
}

pub struct Test262Suite<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> Test262Suite<T> {
    pub fn new() -> Self {
        Self { test_root: project_root().join(FIXTURES_PATH), test_cases: vec![] }
    }
}

impl<T: Case> Suite<T> for Test262Suite<T> {
    fn get_test_root(&self) -> &Path {
        &self.test_root
    }

    fn skip_test_path(&self, path: &Path) -> bool {
        let path = path.to_string_lossy();
        // ignore markdown files
        path.ends_with(".md") ||
        // ignore fixtures
        path.contains("_FIXTURE") ||
        // ignore regexp as we don't have a regexp parser for now
        (path.contains("literals") && path.contains("regexp"))
    }

    fn save_test_cases(&mut self, cases: Vec<T>) {
        self.test_cases = cases;
    }

    fn get_test_cases(&self) -> &Vec<T> {
        &self.test_cases
    }
    fn get_test_cases_mut(&mut self) -> &mut Vec<T> {
        &mut self.test_cases
    }
}

pub struct Test262Case {
    path: PathBuf,
    code: String,
    meta: MetaData,
    should_fail: bool,
    result: TestResult,
}

impl Test262Case {
    pub fn meta(&self) -> &MetaData {
        &self.meta
    }

    /// # Errors
    /// # Panics
    pub fn read_metadata(code: &str) -> io::Result<MetaData> {
        let (start, end) = (code.find("/*---").unwrap(), code.find("---*/").unwrap());
        let yaml = &code[start + 5..end].replace('\r', "\n");
        serde_yaml::from_str(yaml).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn set_result(&mut self, result: TestResult) {
        self.result = result;
    }

    fn compute_should_fail(meta: &MetaData) -> bool {
        meta.negative.as_ref().filter(|n| n.phase == Phase::Parse).is_some()
    }
}

impl Case for Test262Case {
    fn new(path: PathBuf, code: String) -> Self {
        let meta = Self::read_metadata(&code).expect("read test262 yaml meta");
        let should_fail = Self::compute_should_fail(&meta);
        Self { path, code, meta, should_fail, result: TestResult::ToBeRun }
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn test_result(&self) -> &TestResult {
        &self.result
    }

    fn should_fail(&self) -> bool {
        self.should_fail
    }

    fn skip_test_case(&self) -> bool {
        [
            // Regex parser is required. See https://github.com/oxc-project/oxc/issues/385#issuecomment-1755566240
            "regexp-v-flag",
            "regexp-unicode-property-escapes",
        ]
        .iter()
        .any(|feature| self.meta.features.iter().any(|f| **f == **feature))
    }

    // Unless configured otherwise (via the noStrict, onlyStrict, module, or raw flags),
    // each test must be executed twice: once in ECMAScript's non-strict mode, and again in ECMAScript's strict mode.
    // To run in strict mode, the test contents must be modified prior to execution--
    // a "use strict" directive must be inserted as the initial character sequence of the file
    // https://github.com/tc39/test262/blob/main/INTERPRETING.md#strict-mode
    fn run(&mut self) {
        let flags = &self.meta.flags;
        let source_type = SourceType::default().with_script(true);

        self.result = if flags.contains(&TestFlag::OnlyStrict) {
            self.execute(source_type.with_always_strict(true))
        } else if flags.contains(&TestFlag::Module) {
            self.execute(source_type.with_module(true))
        } else if flags.contains(&TestFlag::NoStrict) || flags.contains(&TestFlag::Raw) {
            self.execute(source_type)
        } else {
            let res = self.execute(source_type.with_always_strict(true));
            if matches!(res, TestResult::Passed) {
                self.execute(source_type.with_always_strict(false))
            } else {
                res
            }
        };
    }

    fn check_semantic(&self, semantic: &oxc_semantic::Semantic<'_>) -> Option<TestResult> {
        if are_all_identifiers_resolved(semantic) {
            None
        } else {
            Some(TestResult::ParseError("Unset symbol / reference".to_string(), true))
        }
    }
}

fn are_all_identifiers_resolved(semantic: &oxc_semantic::Semantic<'_>) -> bool {
    use oxc_ast::AstKind;
    use oxc_semantic::AstNode;

    let ast_nodes = semantic.nodes();
    let has_non_resolved = ast_nodes.iter().any(|node| {
        match node.kind() {
            AstKind::BindingIdentifier(id) => {
                let mut parents = ast_nodes.iter_parents(node.id()).map(AstNode::kind);
                parents.next(); // Exclude BindingIdentifier itself
                if let (Some(AstKind::Function(_)), Some(AstKind::IfStatement(_))) =
                    (parents.next(), parents.next())
                {
                    return false;
                }
                id.symbol_id.get().is_none()
            }
            AstKind::IdentifierReference(ref_id) => ref_id.reference_id.get().is_none(),
            _ => false,
        }
    });

    !has_non_resolved
}
