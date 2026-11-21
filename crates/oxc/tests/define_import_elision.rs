//! `define` runs after TypeScript import elision. The DCE pass that follows can drop the only
//! value use of an imported binding and leave `import "mod"`. These cases check that TS input
//! without `onlyRemoveTypeImports` elides that import, while real side-effect imports and JS
//! inputs keep the module load.

use std::path::Path;

use oxc::{
    CompilerInterface, codegen::CodegenReturn, span::SourceType, transformer::TransformOptions,
    transformer_plugins::ReplaceGlobalDefinesConfig,
};

struct Harness {
    options: TransformOptions,
    define: ReplaceGlobalDefinesConfig,
    code: String,
}

impl CompilerInterface for Harness {
    fn transform_options(&self) -> Option<&TransformOptions> {
        Some(&self.options)
    }

    fn define_options(&self) -> Option<ReplaceGlobalDefinesConfig> {
        Some(self.define.clone())
    }

    fn check_semantic_error(&self) -> bool {
        false
    }

    fn after_codegen(&mut self, ret: CodegenReturn<'_>) {
        self.code = ret.code;
    }
}

fn compile(source: &str, source_type: SourceType, only_remove_type_imports: bool) -> String {
    let define = ReplaceGlobalDefinesConfig::new(&[("import.meta.env.DEV", "false")]).unwrap();
    let mut options = TransformOptions::default();
    options.typescript.only_remove_type_imports = only_remove_type_imports;
    let mut harness = Harness { options, define, code: String::new() };
    harness.compile(source, source_type, Path::new("fixture.ts"));
    harness.code
}

fn assert_code(
    source: &str,
    source_type: SourceType,
    only_remove_type_imports: bool,
    expected: &str,
) {
    let code = compile(source, source_type, only_remove_type_imports);
    assert_eq!(code, expected, "source:\n{source}");
}

#[test]
fn define_elides_ts_import_whose_only_value_use_was_folded_away() {
    let source = "\
import { devGate } from '#dev-gate';
export function warn(msg: string): void {
  if (!import.meta.env.DEV || !devGate()) return;
  console.warn(msg);
}
";
    assert_code(source, SourceType::ts(), false, "export function warn(msg) {}\n");
}

#[test]
fn define_keeps_an_explicit_side_effect_import() {
    let source = "\
import '#side';
import { devGate } from '#dev-gate';
export function warn(msg: string): void {
  if (!import.meta.env.DEV || !devGate()) return;
  console.warn(msg);
}
";
    assert_code(
        source,
        SourceType::ts(),
        false,
        "import \"#side\";\nexport function warn(msg) {}\n",
    );
}

#[test]
fn define_keeps_side_effect_import_merged_with_a_now_unused_binding() {
    let source = "\
import '#mod';
import { devGate } from '#mod';
export function warn(msg: string): void {
  if (!import.meta.env.DEV || !devGate()) return;
  console.warn(msg);
}
";
    let code = compile(source, SourceType::ts(), false);
    assert!(code.contains("import \"#mod\";"), "{code}");
    assert!(!code.contains("devGate"), "{code}");
}

#[test]
fn define_keeps_a_binding_that_is_still_used() {
    let source = "\
import { devGate, keep } from '#mod';
export function warn(msg: string): void {
  if (import.meta.env.DEV) devGate();
  keep();
}
";
    assert_code(
        source,
        SourceType::ts(),
        false,
        "import { keep } from \"#mod\";\nexport function warn(msg) {\n\tkeep();\n}\n",
    );
}

#[test]
fn define_preserves_module_kind_when_the_import_was_the_only_module_syntax() {
    let source = "\
import { devGate } from '#dev-gate';
this.x = 1;
if (!import.meta.env.DEV || devGate()) {}
";
    assert_code(source, SourceType::ts(), false, "this.x = 1;\nexport {};\n");
}

#[test]
fn define_keeps_bare_import_when_only_type_imports_are_removed() {
    let source = "\
import { devGate } from '#dev-gate';
export function warn(msg: string): void {
  if (!import.meta.env.DEV || !devGate()) return;
  console.warn(msg);
}
";
    assert_code(
        source,
        SourceType::ts(),
        true,
        "import \"#dev-gate\";\nexport function warn(msg) {}\n",
    );
}

#[test]
fn define_keeps_bare_import_for_javascript() {
    let source = "\
import { devGate } from '#dev-gate';
export function warn(msg) {
  if (!import.meta.env.DEV || !devGate()) return;
  console.warn(msg);
}
";
    assert_code(
        source,
        SourceType::mjs(),
        false,
        "import \"#dev-gate\";\nexport function warn(msg) {}\n",
    );
}

#[test]
fn ts_elides_an_import_that_was_already_unused_in_the_source() {
    let source = "\
import { devGate } from '#dev-gate';
export const x = 1;
";
    assert_code(source, SourceType::ts(), false, "export const x = 1;\n");
}
