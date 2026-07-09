//! Integration tests for tsconfig parsing: `extends` chains resolved and merged with tsc's
//! semantics, `${configDir}` substitution, and the parsed config driving a `Program`.

use std::path::{Path, PathBuf};

use oxc_type_checker::{
    compiler::{Program, ProgramOptions},
    core::{ModuleResolutionKind, ScriptTarget},
    tsoptions::{get_file_names, parse_config_file},
};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tsoptions").join(name)
}

#[test]
fn extends_chain_merges_with_tsc_semantics() {
    let dir = fixture("extends-chain");
    let config = parse_config_file(&dir.join("tsconfig.json")).unwrap();
    let options = &config.compiler_options;

    // Inherited from the base config (which also proves JSONC comments/trailing commas parse).
    assert_eq!(options.strict, Some(true));
    assert_eq!(options.lib.as_deref(), Some(&["es2020".to_string()][..]));
    // The child's own value wins over the inherited one.
    assert_eq!(options.target, Some(ScriptTarget::Es2022));
    // The child's explicit `"outDir": null` resets the inherited value instead of inheriting.
    assert_eq!(options.out_dir, None);
    // Path options anchor to the directory of the config that DEFINED them (the base's).
    assert_eq!(options.declaration_dir, Some(dir.join("config/types")));
    // `${configDir}` resolves to the ROOT config's directory, even when written in the base.
    assert_eq!(options.type_roots, Some(vec![dir.join("typings")]));
    // `paths` values are kept as written; their base path is the defining config's directory.
    assert_eq!(options.paths_base_path, Some(dir.join("config")));
    // Inherited `include` stays anchored to the defining config's directory; the own `exclude`
    // anchors to the root config's directory.
    assert_eq!(config.include, Some(vec![dir.join("config/src/**/*")]));
    assert_eq!(config.exclude, Some(vec![dir.join("dist")]));
    assert_eq!(options.config_file_path, Some(dir.join("tsconfig.json")));

    // The file-spec expansion sees through all of it: the base's `include` picks up the file
    // that lives next to the base config.
    let file_names = get_file_names(&config);
    assert_eq!(file_names, vec![dir.join("config/src/main.ts")]);
}

#[test]
fn extends_array_later_entries_win() {
    let dir = fixture("extends-array");
    let config = parse_config_file(&dir.join("tsconfig.json")).unwrap();
    let options = &config.compiler_options;
    assert_eq!(options.target, Some(ScriptTarget::Es2020));
    assert_eq!(options.strict, Some(true));
}

#[test]
fn extends_cycle_is_an_error() {
    let dir = fixture("extends-cycle");
    let error = parse_config_file(&dir.join("a.json")).unwrap_err();
    assert!(error.to_string().contains("Circularity detected"), "unexpected error: {error:#}");
}

#[test]
fn extends_resolves_bare_specifiers_through_node_modules() {
    let dir = fixture("extends-package");
    // `node_modules/<pkg>/tsconfig.json` for a bare package specifier.
    let config = parse_config_file(&dir.join("tsconfig.json")).unwrap();
    assert_eq!(config.compiler_options.module_resolution, Some(ModuleResolutionKind::Bundler));
    // The package.json `tsconfig` field names the config to use.
    let config = parse_config_file(&dir.join("tsconfig-field.json")).unwrap();
    assert_eq!(config.compiler_options.strict, Some(true));
}

#[test]
fn parsed_config_drives_a_program() {
    let dir = fixture("program-e2e");
    let config = parse_config_file(&dir.join("tsconfig.json")).unwrap();
    let root_files = get_file_names(&config);
    let program = Program::new(ProgramOptions {
        current_directory: dir.clone(),
        root_files,
        config: Some(config),
    });
    let file_names: Vec<PathBuf> =
        program.files().iter().map(|file| file.file_name().to_path_buf()).collect();
    assert!(file_names.contains(&dir.join("src/index.ts")), "missing index.ts: {file_names:?}");
    assert!(file_names.contains(&dir.join("src/util.ts")), "missing util.ts: {file_names:?}");
    assert!(program.missing_files().is_empty());
}
