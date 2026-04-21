use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use cow_utils::CowUtils;

use encoding_rs::UTF_16LE;
use encoding_rs_io::DecodeReaderBytesBuilder;
use oxc::{span::SourceType, transformer::BabelOptions};
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::{
    AcornJsxFile, BabelFile, MiscFile, Test262File, TestData, TypeScriptFile, babel, test262,
    typescript, workspace_root,
};

impl TestData {
    pub fn load(filter: Option<&str>) -> Self {
        let ((test262, babel), (typescript, (misc, acorn_jsx))) = rayon::join(
            || rayon::join(|| load_test262(filter), || load_babel(filter)),
            || {
                rayon::join(
                    || load_typescript(filter),
                    || rayon::join(|| load_misc(filter), || load_acorn_jsx(filter)),
                )
            },
        );
        Self { test262, babel, typescript, misc, acorn_jsx }
    }
}

fn walk_and_read(
    root: &Path,
    filter: Option<&str>,
    skip_path: impl Fn(&Path) -> bool + Sync,
) -> Vec<(PathBuf, String)> {
    let base = workspace_root();
    let full_root = base.join(root);

    let paths: Vec<_> = WalkDir::new(&full_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
        .filter(|e| e.file_name() != ".DS_Store")
        .map(|e| e.path().to_owned())
        .filter(|path| !skip_path(path))
        .filter(|path| filter.is_none_or(|q| path.to_string_lossy().contains(q)))
        .collect();

    if paths.is_empty() && filter.is_none() {
        println!("-------------------------------------------------------");
        println!("git submodule is empty for {}", root.display());
        println!("Running `just submodules` to clone the submodules");
        println!("-------------------------------------------------------");
        Command::new("just")
            .args(["submodules"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("failed to execute `just submodules`");
        return walk_and_read(root, filter, skip_path);
    }

    paths
        .into_par_iter()
        .filter_map(|path| {
            let code = fs::read_to_string(&path).ok().or_else(|| {
                let file = fs::File::open(&path).ok()?;
                let mut content = String::new();
                DecodeReaderBytesBuilder::new()
                    .encoding(Some(UTF_16LE))
                    .build(file)
                    .read_to_string(&mut content)
                    .ok()?;
                Some(content)
            })?;
            // Remove BOM without reallocating
            let mut code = code;
            if code.starts_with('\u{feff}') {
                code.drain(..'\u{feff}'.len_utf8());
            }
            let rel_path = path.strip_prefix(&base).unwrap().to_owned();
            Some((rel_path, code))
        })
        .collect()
}

fn load_test262(filter: Option<&str>) -> Vec<Test262File> {
    let skip_path = |path: &Path| {
        let s = path.to_string_lossy();
        let s = s.cow_replace('\\', "/");
        s.contains("test262/test/staging")
            || path.extension().is_some_and(|e| e.eq_ignore_ascii_case("md"))
            || s.contains("_FIXTURE")
            || s.contains("annexB/language/expressions/assignmenttargettype")
    };

    walk_and_read(Path::new("test262/test"), filter, skip_path)
        .into_par_iter()
        .map(|(path, code)| {
            let meta = test262::parse_meta(&code);
            Test262File { path, code, meta }
        })
        .collect()
}

fn load_babel(filter: Option<&str>) -> Vec<BabelFile> {
    let skip_path = |path: &Path| {
        let s = path.to_string_lossy();
        let s = s.cow_replace('\\', "/");
        let not_supported = [
            "experimental",
            "record-and-tuple",
            "es-record",
            "es-tuple",
            "with-pipeline",
            "v8intrinsic",
            "async-do-expression",
            "export-ns-from",
            "annex-b/disabled",
            "annex-b/enabled/valid-assignment-target-type",
            "module-block",
            "typescript/arrow-function/arrow-like-in-conditional-2",
            "typescript/cast/satisfies-const-error",
            "es2022/top-level-await-unambiguous",
            "explicit-resource-management/valid-for-await-using-binding-escaped-of-of",
            "explicit-resource-management/valid-for-using-binding-escaped-of-of",
        ]
        .iter()
        .any(|p| s.contains(p));
        let not_interesting = [
            "core/categorized/invalid-startindex-and-startline-specified-without-startcolumn",
            "core/categorized/startline-and-startcolumn-specified",
            "core/categorized/startline-specified",
            "core/sourcetype-commonjs/invalid-allowAwaitOutsideFunction-false",
            "core/sourcetype-commonjs/invalid-allowNewTargetOutsideFunction-false",
            "core/sourcetype-commonjs/invalid-allowNewTargetOutsideFunction-true",
            "core/sourcetype-commonjs/invalid-allowReturnOutsideFunction-false",
            "core/sourcetype-commonjs/invalid-allowReturnOutsideFunction-true",
        ]
        .iter()
        .any(|p| s.strip_suffix("/input.js").is_some_and(|s| s.ends_with(p)));
        let bad_ext = path.extension().is_none_or(|ext| ext == "json" || ext == "md");
        not_supported || not_interesting || bad_ext
    };

    walk_and_read(Path::new("babel/packages/babel-parser/test/fixtures"), filter, skip_path)
        .into_par_iter()
        .filter_map(|(path, code)| {
            let dir = workspace_root().join(&path);
            let options = BabelOptions::from_test_path(dir.parent()?);

            // Skip unsupported plugins
            let not_supported_plugins = [
                "async-do-expression",
                "flow",
                "placeholders",
                "decorators-legacy",
                "recordAndTuple",
            ];
            let skip = options
                .plugins
                .unsupported
                .iter()
                .any(|p| not_supported_plugins.contains(&p.as_str()))
                || options.allow_await_outside_function
                || options.allow_undeclared_exports
                || options.allow_new_target_outside_function
                || options.allow_super_outside_method
                || options.has_disallow_ambiguous_jsx_like();
            if skip {
                return None;
            }

            let mut source_type = SourceType::from_path(&path)
                .ok()?
                .with_script(true)
                .with_jsx(options.is_jsx())
                .with_typescript(options.is_typescript())
                .with_typescript_definition(options.is_typescript_definition());
            if options.is_unambiguous() {
                source_type = source_type.with_unambiguous(true);
            } else if options.is_module() {
                source_type = source_type.with_module(true);
            } else if options.is_commonjs() {
                source_type = source_type.with_commonjs(true);
            }

            let should_fail = babel::determine_should_fail(&path, &options);
            Some(BabelFile { path, code, source_type, options, should_fail })
        })
        .collect()
}

fn load_typescript(filter: Option<&str>) -> Vec<TypeScriptFile> {
    let skip_path = |path: &Path| {
        let s = path.to_string_lossy();
        #[cfg(any(coverage, coverage_nightly))]
        let supported = ["conformance"].iter().any(|p| s.contains(p));
        #[cfg(not(any(coverage, coverage_nightly)))]
        let supported = ["conformance", "compiler"].iter().any(|p| s.contains(p));
        let unsupported =
            typescript::constants::NOT_SUPPORTED_TEST_PATHS.iter().any(|p| s.contains(p));
        !supported || unsupported
    };

    walk_and_read(Path::new("typescript/tests/cases"), filter, skip_path)
        .into_par_iter()
        .map(|(path, code)| {
            let content = typescript::meta::TestCaseContent::make_units_from_test(&path, &code);
            let should_fail = content
                .error_codes
                .iter()
                .any(|c| !typescript::constants::NOT_SUPPORTED_ERROR_CODES.contains(c.as_str()));
            TypeScriptFile {
                path,
                code,
                units: content.tests,
                settings: content.settings,
                should_fail,
                error_codes: content.error_codes,
            }
        })
        .collect()
}

fn load_misc(filter: Option<&str>) -> Vec<MiscFile> {
    let skip_path = |_: &Path| false;

    let mut files: Vec<_> = walk_and_read(Path::new("misc"), filter, skip_path)
        .into_par_iter()
        .filter_map(|(path, code)| {
            let should_fail = path.to_string_lossy().contains("fail");
            let source_type = SourceType::from_path(&path).ok()?;
            Some(MiscFile { path, code, source_type, should_fail })
        })
        .collect();

    // Add extra generated cases
    if filter.is_none() {
        let code = String::from("a") + &"+ a".repeat(1000);
        files.push(MiscFile {
            path: PathBuf::from("huge_binary_expression.js"),
            code,
            source_type: SourceType::cjs(),
            should_fail: false,
        });

        let code = "if (true) {".repeat(1000) + &"}".repeat(1000);
        files.push(MiscFile {
            path: PathBuf::from("huge_nested_statements.js"),
            code,
            source_type: SourceType::cjs(),
            should_fail: false,
        });
    }

    files
}

fn load_acorn_jsx(filter: Option<&str>) -> Vec<AcornJsxFile> {
    let skip_path = |path: &Path| path.extension().is_none_or(|ext| ext != "jsx");

    walk_and_read(Path::new("estree-conformance/tests/acorn-jsx"), filter, skip_path)
        .into_par_iter()
        .map(|(path, code)| {
            let should_fail =
                path.parent().and_then(Path::file_name).is_some_and(|name| name == "fail");
            AcornJsxFile { path, code, should_fail }
        })
        .collect()
}
