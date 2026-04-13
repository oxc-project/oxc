use std::{
    fmt, fs, io,
    io::Write,
    path::{Path, PathBuf},
};

use proc_macro2::Span;
use syn::{
    Attribute, Expr, Ident, LitStr, Token,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    visit::Visit,
};
use tempfile::NamedTempFile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewriteReport {
    pub rewritten_rules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleVersionViolation {
    pub path: PathBuf,
    pub rule_name: String,
    pub category: String,
    pub kind: RuleVersionViolationKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleVersionViolationKind {
    Next,
    Invalid(String),
}

impl fmt::Display for RuleVersionViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            RuleVersionViolationKind::Next => write!(
                f,
                "{}: {} in category `{}` still uses version = \"next\"",
                self.path.display(),
                self.rule_name,
                self.category
            ),
            RuleVersionViolationKind::Invalid(version) => write!(
                f,
                "{}: {} in category `{}` uses invalid version `{version}`",
                self.path.display(),
                self.rule_name,
                self.category
            ),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    InvalidReleaseVersion(String),
    Parse { path: PathBuf, message: String },
    Symlink { path: PathBuf },
    PathOutsideRules { path: PathBuf, rules_root: PathBuf },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::InvalidReleaseVersion(version) => {
                write!(f, "release version must be a real version, got `{version}`")
            }
            Self::Parse { path, message } => {
                write!(
                    f,
                    "{}: failed to parse declare_oxc_lint! metadata: {message}",
                    path.display()
                )
            }
            Self::Symlink { path } => {
                write!(f, "{}: symlinks are not supported in the lint rules tree", path.display())
            }
            Self::PathOutsideRules { path, rules_root } => write!(
                f,
                "{}: resolved path is outside the lint rules tree {}",
                path.display(),
                rules_root.display()
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub fn rewrite_rule_versions(root: &Path, _release_version: &str) -> Result<RewriteReport, Error> {
    let release_version = validate_release_version(_release_version)?;
    let rules_root = canonical_rules_root(root)?;
    let mut rewritten_rules = Vec::new();
    let mut file_rewrites = Vec::new();
    let mut prepared_file_rewrites = Vec::new();

    for path in collect_rule_files(&rules_root)? {
        let source = fs::read_to_string(&path)?;
        let occurrences = parse_rule_macros(&source, &path)?;

        let rewrites = occurrences
            .iter()
            .filter_map(|occurrence| occurrence.rewrite_range_for(release_version))
            .collect::<Vec<_>>();
        if rewrites.is_empty() {
            continue;
        }

        file_rewrites.push(FileRewrite::new(path, source, rewrites));
    }

    for file_rewrite in file_rewrites {
        ensure_path_is_within_rules_root(&file_rewrite.path, &rules_root)?;
        prepared_file_rewrites.push(PreparedFileRewrite::new(file_rewrite)?);
    }

    for prepared_file_rewrite in prepared_file_rewrites {
        rewritten_rules.extend(prepared_file_rewrite.persist()?);
    }

    Ok(RewriteReport { rewritten_rules })
}

pub fn check_rule_versions(root: &Path) -> Result<Vec<RuleVersionViolation>, Error> {
    let rules_root = canonical_rules_root(root)?;
    let mut violations = Vec::new();

    for path in collect_rule_files(&rules_root)? {
        let source = fs::read_to_string(&path)?;
        let occurrences = parse_rule_macros(&source, &path)?;
        violations
            .extend(occurrences.into_iter().filter_map(|occurrence| occurrence.violation(&path)));
    }

    Ok(violations)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ByteRange {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Rewrite {
    rule_name: String,
    version_range: ByteRange,
    replacement: String,
}

struct FileRewrite {
    path: PathBuf,
    rewritten_source: String,
    rewritten_rules: Vec<String>,
}

impl FileRewrite {
    fn new(path: PathBuf, source: String, rewrites: Vec<Rewrite>) -> Self {
        let mut rewritten_source = source;
        for rewrite in rewrites.iter().rev() {
            rewritten_source.replace_range(
                rewrite.version_range.start..rewrite.version_range.end,
                rewrite.replacement.as_str(),
            );
        }

        Self {
            path,
            rewritten_source,
            rewritten_rules: rewrites.into_iter().map(|rewrite| rewrite.rule_name).collect(),
        }
    }
}

struct PreparedFileRewrite {
    path: PathBuf,
    temp_file: NamedTempFile,
    rewritten_rules: Vec<String>,
}

impl PreparedFileRewrite {
    fn new(file_rewrite: FileRewrite) -> Result<Self, Error> {
        let Some(parent) = file_rewrite.path.parent() else {
            return Err(Error::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} has no parent directory", file_rewrite.path.display()),
            )));
        };

        let mut temp_file = NamedTempFile::new_in(parent)?;
        temp_file.write_all(file_rewrite.rewritten_source.as_bytes())?;
        temp_file.as_file_mut().sync_all()?;

        Ok(Self {
            path: file_rewrite.path,
            temp_file,
            rewritten_rules: file_rewrite.rewritten_rules,
        })
    }

    fn persist(self) -> Result<Vec<String>, Error> {
        self.temp_file.persist(&self.path).map_err(|error| Error::Io(error.error))?;
        Ok(self.rewritten_rules)
    }
}

#[derive(Debug)]
struct RuleMacroOccurrence {
    rule_name: String,
    category: String,
    version: Option<VersionLiteral>,
}

impl RuleMacroOccurrence {
    fn rewrite_range_for(&self, release_version: &str) -> Option<Rewrite> {
        let version = self.version.as_ref()?;
        if version.value != "next" || is_nursery_exempt(&self.category) {
            return None;
        }

        Some(Rewrite {
            rule_name: self.rule_name.clone(),
            version_range: version.range,
            replacement: format!("\"{release_version}\""),
        })
    }

    fn violation(&self, path: &Path) -> Option<RuleVersionViolation> {
        if is_nursery_exempt(&self.category) {
            return None;
        }

        let kind = match self.version.as_ref() {
            None => return None,
            Some(version) if version.value == "next" => RuleVersionViolationKind::Next,
            Some(version) if !is_valid_release_version(&version.value) => {
                RuleVersionViolationKind::Invalid(version.value.clone())
            }
            Some(_) => return None,
        };

        Some(RuleVersionViolation {
            path: path.to_path_buf(),
            rule_name: self.rule_name.clone(),
            category: self.category.clone(),
            kind,
        })
    }
}

#[derive(Debug)]
struct VersionLiteral {
    value: String,
    range: ByteRange,
}

struct RuleDeclaration {
    name: Ident,
    category: Ident,
    version: Option<LitStr>,
}

impl Parse for RuleDeclaration {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _ = input.call(Attribute::parse_outer)?;

        let name: Ident = input.parse()?;

        if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            let _: Ident = content.parse()?;
        }

        input.parse::<Token!(,)>()?;
        let _: Ident = input.parse()?;
        input.parse::<Token!(,)>()?;
        let category: Ident = input.parse()?;

        let mut version = None;

        while input.peek(Token!(,)) {
            input.parse::<Token!(,)>()?;
            if input.is_empty() {
                break;
            }

            let key: Ident = input.parse()?;
            if input.peek(Token!(=)) {
                input.parse::<Token!(=)>()?;
                if key == "version" {
                    version = Some(input.parse()?);
                } else {
                    let _: Expr = input.parse()?;
                }
            }
        }

        let remaining = input.parse::<proc_macro2::TokenStream>()?;
        if !remaining.is_empty() {
            return Err(syn::Error::new(remaining.span(), "unexpected trailing tokens"));
        }

        Ok(Self { name, category, version })
    }
}

struct RuleMacroCollector<'a> {
    source: &'a str,
    occurrences: Vec<RuleMacroOccurrence>,
    errors: Vec<String>,
}

impl<'ast> Visit<'ast> for RuleMacroCollector<'_> {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        if item.path.segments.last().is_some_and(|segment| segment.ident == "declare_oxc_lint") {
            match syn::parse2::<RuleDeclaration>(item.tokens.clone()) {
                Ok(declaration) => {
                    let version = declaration.version.as_ref().and_then(|literal| {
                        span_to_range(self.source, literal.span())
                            .map(|range| VersionLiteral { value: literal.value(), range })
                    });

                    self.occurrences.push(RuleMacroOccurrence {
                        rule_name: declaration.name.to_string(),
                        category: declaration.category.to_string(),
                        version,
                    });
                }
                Err(error) => self.errors.push(error.to_string()),
            }
        }

        syn::visit::visit_macro(self, item);
    }
}

fn validate_release_version(release_version: &str) -> Result<&str, Error> {
    if !is_valid_release_version(release_version) {
        return Err(Error::InvalidReleaseVersion(release_version.to_string()));
    }
    Ok(release_version)
}

fn canonical_rules_root(root: &Path) -> Result<PathBuf, Error> {
    let rules_root = root.join("crates/oxc_linter/src/rules");
    let metadata = fs::symlink_metadata(&rules_root)?;
    if metadata.file_type().is_symlink() {
        return Err(Error::Symlink { path: rules_root });
    }
    fs::canonicalize(rules_root).map_err(Error::from)
}

fn collect_rule_files(rules_root: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut files = Vec::new();
    collect_rule_files_in(rules_root, rules_root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_rule_files_in(
    rules_root: &Path,
    dir: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_symlink() {
            return Err(Error::Symlink { path });
        }

        if file_type.is_dir() {
            ensure_path_is_within_rules_root(&path, rules_root)?;
            collect_rule_files_in(rules_root, &path, files)?;
        } else if file_type.is_file() && path.extension().is_some_and(|extension| extension == "rs")
        {
            ensure_path_is_within_rules_root(&path, rules_root)?;
            files.push(path);
        }
    }

    Ok(())
}

fn parse_rule_macros(source: &str, path: &Path) -> Result<Vec<RuleMacroOccurrence>, Error> {
    let file = syn::parse_file(source)
        .map_err(|error| Error::Parse { path: path.to_path_buf(), message: error.to_string() })?;
    let mut collector = RuleMacroCollector { source, occurrences: Vec::new(), errors: Vec::new() };
    collector.visit_file(&file);

    if collector.errors.is_empty() {
        Ok(collector.occurrences)
    } else {
        Err(Error::Parse { path: path.to_path_buf(), message: collector.errors.join("; ") })
    }
}

fn span_to_range(source: &str, span: Span) -> Option<ByteRange> {
    let start = span.start();
    let end = span.end();
    Some(ByteRange {
        start: line_col_to_offset(source, start.line, start.column)?,
        end: line_col_to_offset(source, end.line, end.column)?,
    })
}

fn line_col_to_offset(content: &str, line: usize, column: usize) -> Option<usize> {
    let mut current_line = 1;
    let mut line_start = 0;

    for (index, ch) in content.char_indices() {
        if current_line == line {
            let line_content = &content[line_start..];
            let column_offset: usize = line_content.chars().take(column).map(char::len_utf8).sum();
            return Some(line_start + column_offset);
        }
        if ch == '\n' {
            current_line += 1;
            line_start = index + 1;
        }
    }

    if current_line == line {
        let line_content = &content[line_start..];
        let column_offset: usize = line_content.chars().take(column).map(char::len_utf8).sum();
        return Some(line_start + column_offset);
    }

    None
}

fn ensure_path_is_within_rules_root(path: &Path, rules_root: &Path) -> Result<(), Error> {
    let canonical_path = fs::canonicalize(path)?;
    if canonical_path.starts_with(rules_root) {
        return Ok(());
    }

    Err(Error::PathOutsideRules { path: path.to_path_buf(), rules_root: rules_root.to_path_buf() })
}

fn is_nursery_exempt(category: &str) -> bool {
    // Nursery rules intentionally stay on `version = "next"` until they move to a stable category.
    category == "nursery"
}

fn is_valid_release_version(version: &str) -> bool {
    let mut parts = version.split('.');
    let (Some(major), Some(minor), Some(patch), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };

    [major, minor, patch]
        .into_iter()
        .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

#[cfg(test)]
mod test {
    use super::*;
    use oxc_tasks_common::project_root;
    use tempfile::TempDir;

    #[test]
    fn prepare_release_workflow_validates_after_rewrite() {
        let workflow =
            fs::read_to_string(project_root().join(".github/workflows/prepare_release_apps.yml"))
                .unwrap();
        let rewrite_step = workflow.find("Rewrite rule versions").unwrap();
        let check_step = workflow.find("oxc_release_rule_versions -- check").unwrap();

        assert!(check_step > rewrite_step);
    }

    #[test]
    fn rewrite_updates_stable_next_versions() {
        let dir = create_test_rules_dir();
        let stable_rule = dir.path().join("crates/oxc_linter/src/rules/eslint/no_debugger.rs");
        write_rule(
            &stable_rule,
            r#"
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// docs
    NoDebugger,
    eslint,
    correctness,
    version = "next",
);
"#,
        );

        let report = rewrite_rule_versions(dir.path(), "1.61.0").unwrap();
        let updated = fs::read_to_string(stable_rule).unwrap();

        assert_eq!(report.rewritten_rules, ["NoDebugger"]);
        assert!(updated.contains(r#"version = "1.61.0""#));
        assert!(!updated.contains(r#"version = "next""#));
    }

    #[test]
    fn rewrite_keeps_nursery_next_versions() {
        let dir = create_test_rules_dir();
        let nursery_rule = dir.path().join("crates/oxc_linter/src/rules/eslint/no_debugger.rs");
        write_rule(
            &nursery_rule,
            r#"
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// docs
    NoDebugger,
    eslint,
    nursery,
    version = "next",
);
"#,
        );

        let report = rewrite_rule_versions(dir.path(), "1.61.0").unwrap();
        let updated = fs::read_to_string(nursery_rule).unwrap();

        assert!(report.rewritten_rules.is_empty());
        assert!(updated.contains(r#"version = "next""#));
    }

    #[test]
    fn check_only_reports_stable_next_versions() {
        let dir = create_test_rules_dir();
        write_rule(
            &dir.path().join("crates/oxc_linter/src/rules/eslint/no_debugger.rs"),
            r#"
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// docs
    NoDebugger,
    eslint,
    correctness,
    version = "next",
);
"#,
        );
        write_rule(
            &dir.path().join("crates/oxc_linter/src/rules/eslint/no_alert.rs"),
            r#"
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// docs
    NoAlert,
    eslint,
    nursery,
    version = "next",
);
"#,
        );

        let violations = check_rule_versions(dir.path()).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_name, "NoDebugger");
        assert_eq!(violations[0].category, "correctness");
        assert_eq!(violations[0].kind, RuleVersionViolationKind::Next);
    }

    #[test]
    fn rewrite_rejects_invalid_release_versions() {
        let dir = create_test_rules_dir();

        let error = rewrite_rule_versions(dir.path(), "foo").unwrap_err();

        assert!(matches!(error, Error::InvalidReleaseVersion(version) if version == "foo"));
    }

    #[test]
    fn check_reports_invalid_stable_versions() {
        let dir = create_test_rules_dir();
        write_rule(
            &dir.path().join("crates/oxc_linter/src/rules/eslint/no_debugger.rs"),
            r#"
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// docs
    NoDebugger,
    eslint,
    correctness,
    version = "foo",
);
"#,
        );

        let violations = check_rule_versions(dir.path()).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].kind, RuleVersionViolationKind::Invalid("foo".to_string()));
    }

    #[test]
    fn rewrite_is_atomic_when_a_later_file_fails_to_parse() {
        let dir = create_test_rules_dir();
        let stable_rule = dir.path().join("crates/oxc_linter/src/rules/eslint/a_good.rs");
        let invalid_rule = dir.path().join("crates/oxc_linter/src/rules/eslint/b_bad.rs");
        write_rule(
            &stable_rule,
            r#"
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// docs
    NoDebugger,
    eslint,
    correctness,
    version = "next",
);
"#,
        );
        write_rule(
            &invalid_rule,
            r#"
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// docs
    BrokenRule,
    eslint,
    correctness,
    version = "next"
"#,
        );

        let error = rewrite_rule_versions(dir.path(), "1.61.0").unwrap_err();
        let stable_rule_after = fs::read_to_string(stable_rule).unwrap();

        assert!(matches!(error, Error::Parse { .. }));
        assert!(stable_rule_after.contains(r#"version = "next""#));
    }

    #[cfg(unix)]
    #[test]
    fn rewrite_rejects_symlink_entries() {
        use std::os::unix::fs::symlink;

        let dir = create_test_rules_dir();
        let external_rule = dir.path().join("outside.rs");
        let symlink_path = dir.path().join("crates/oxc_linter/src/rules/eslint/symlink_rule.rs");
        write_rule(
            &external_rule,
            r#"
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// docs
    SymlinkRule,
    eslint,
    correctness,
    version = "next",
);
"#,
        );
        symlink(&external_rule, &symlink_path).unwrap();

        let error = rewrite_rule_versions(dir.path(), "1.61.0").unwrap_err();

        assert!(
            matches!(
                error,
                Error::Symlink { ref path }
                    if path.file_name() == symlink_path.file_name()
            ),
            "unexpected error: {error:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn rewrite_does_not_modify_files_when_a_later_write_would_fail() {
        use std::os::unix::fs::PermissionsExt;

        let dir = create_test_rules_dir();
        let first_rule = dir.path().join("crates/oxc_linter/src/rules/eslint/a_first.rs");
        let locked_dir = dir.path().join("crates/oxc_linter/src/rules/locked");
        let locked_rule = locked_dir.join("b_second.rs");

        fs::create_dir_all(&locked_dir).unwrap();
        write_rule(
            &first_rule,
            r#"
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// docs
    FirstRule,
    eslint,
    correctness,
    version = "next",
);
"#,
        );
        write_rule(
            &locked_rule,
            r#"
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// docs
    SecondRule,
    eslint,
    correctness,
    version = "next",
);
"#,
        );

        fs::set_permissions(&locked_rule, fs::Permissions::from_mode(0o444)).unwrap();
        fs::set_permissions(&locked_dir, fs::Permissions::from_mode(0o555)).unwrap();

        let error = rewrite_rule_versions(dir.path(), "1.61.0").unwrap_err();
        let first_rule_after = fs::read_to_string(first_rule).unwrap();

        assert!(matches!(error, Error::Io(_)));
        assert!(
            first_rule_after.contains(r#"version = "next""#),
            "earlier files should remain unchanged when a later write fails"
        );
    }

    fn create_test_rules_dir() -> TempDir {
        let dir = tempfile::Builder::new().prefix("oxc-release-rule-versions-").tempdir().unwrap();
        fs::create_dir_all(dir.path().join("crates/oxc_linter/src/rules/eslint")).unwrap();
        dir
    }

    fn write_rule(path: &Path, source: &str) {
        fs::write(path, source.trim_start()).unwrap();
    }
}
