pub mod apply_renames;
pub mod convert_ast;
pub mod convert_ast_reverse;
pub mod convert_scope;
pub mod diagnostics;
pub mod prefilter;

use apply_renames::build_rename_plan;
use convert_ast::convert_program;
use convert_scope::convert_scope_info;
use diagnostics::compile_result_to_diagnostics;
use prefilter::{has_react_like_functions, has_resource_management_declarations};
use react_compiler::entrypoint::compile_result::LoggerEvent;

// Re-exported so integrations needn't depend on the upstream `react_compiler` crates.
pub use react_compiler::entrypoint::plugin_options::{
    CompilerTarget, DynamicGatingConfig, GatingConfig, PluginOptions,
};
pub use react_compiler_hir::environment_config::EnvironmentConfig;

use rustc_hash::FxHashSet;

/// [`PluginOptions`] with the compiler's standard defaults (it has no `Default`).
/// Override fields with struct-update syntax: `PluginOptions { ..default_plugin_options() }`.
pub fn default_plugin_options() -> PluginOptions {
    PluginOptions {
        should_compile: true,
        enable_reanimated: false,
        is_dev: false,
        filename: None,
        compilation_mode: "infer".to_string(),
        panic_threshold: "none".to_string(),
        target: CompilerTarget::Version("19".to_string()),
        gating: None,
        dynamic_gating: None,
        no_emit: false,
        output_mode: None,
        eslint_suppression_rules: None,
        flow_suppressions: true,
        ignore_use_no_forget: false,
        custom_opt_out_directives: None,
        environment: EnvironmentConfig::default(),
        source_code: None,
        profiling: false,
        debug: false,
    }
}

#[derive(Default)]
pub struct TransformResult<'a> {
    /// Compiled, ready-to-codegen OXC AST; `None` if the compiler made no changes.
    pub program: Option<oxc_ast::ast::Program<'a>>,
    /// Errors and warnings produced by the compile. Errors (e.g. Rules of Hooks
    /// violations) are hard problems in the source; the program is still left
    /// valid. Warnings include bail-outs where the compiler declined to optimize.
    pub diagnostics: oxc_diagnostics::Diagnostics,
    /// Raw structured logger events from the upstream compiler (compile
    /// success/skip/error with memoization stats), for tooling and profiling.
    /// Unlike `diagnostics`, these are not meant for user-facing reporting.
    pub events: Vec<LoggerEvent>,
}

pub struct LintResult {
    /// Errors and warnings produced by the compile.
    pub diagnostics: oxc_diagnostics::Diagnostics,
}

/// Run the React Compiler on a pre-parsed program, building the semantic model
/// internally and returning the result. `program` in the result is `None` when
/// nothing was compiled (no React-like functions, a bail-out, or no changes).
///
/// Must run **first**, on the pristine AST, before any other transform.
pub fn transform<'a>(
    program: &oxc_ast::ast::Program<'a>,
    allocator: &'a oxc_allocator::Allocator,
    options: PluginOptions,
) -> TransformResult<'a> {
    let source_text = program.source_text;

    // Skip files with no React-like functions, unless the mode compiles everything.
    if !matches!(options.compilation_mode.as_str(), "all" | "annotation")
        && !has_react_like_functions(program)
    {
        return TransformResult::default();
    }

    // `using`/`await using` disposal semantics aren't preserved yet — skip the file.
    if has_resource_management_declarations(program) {
        return TransformResult::default();
    }

    let semantic = oxc_semantic::SemanticBuilder::new()
        .with_build_nodes(true)
        .with_enum_eval(true)
        .build(program)
        .semantic;

    let file = convert_program(program, source_text);
    let scope_info = convert_scope_info(&semantic, program);
    let result =
        react_compiler::entrypoint::program::compile_program(file, scope_info.clone(), options);

    let diagnostics = compile_result_to_diagnostics(&result);
    let (program_ast, events, renames) = match result {
        react_compiler::entrypoint::compile_result::CompileResult::Success {
            ast,
            events,
            renames,
            ..
        } => (ast, events, renames),
        react_compiler::entrypoint::compile_result::CompileResult::Error { events, .. } => {
            (None, events, Vec::new())
        }
    };

    // Rename plan maps source positions of uncompiled references to new names.
    let rename_plan = build_rename_plan(&scope_info, &renames);

    let compiled_program = program_ast.map(|file: react_compiler_ast::File| {
        let mut compiled =
            convert_ast_reverse::convert_program_to_oxc_with_source(&file, allocator, source_text);
        compiled.source_type = program.source_type;
        apply_renames::apply_renames(&mut compiled, &rename_plan, allocator);
        preserve_comments(&mut compiled, program, allocator);
        compiled
    });

    TransformResult { program: compiled_program, diagnostics, events }
}

/// Carry over the comments attached to top-level statements of the compiled
/// program, so codegen can re-emit them. The `react_compiler_ast` roundtrip
/// drops comments, so we reuse the ones from the original `source` program
/// (already parsed) rather than re-parsing the source.
fn preserve_comments<'a>(
    compiled: &mut oxc_ast::ast::Program<'a>,
    source: &oxc_ast::ast::Program<'a>,
    allocator: &'a oxc_allocator::Allocator,
) {
    // Keep only comments attached to a top-level statement; inner comments have
    // `attached_to` positions that match no top-level statement.
    let mut top_level_starts = FxHashSet::default();
    top_level_starts.insert(0u32);
    for stmt in &compiled.body {
        use oxc_span::GetSpan;
        let start = stmt.span().start;
        if start > 0 {
            top_level_starts.insert(start);
        }
    }

    // Copy only comments attached to top-level statements.
    let mut comments = oxc_allocator::Vec::with_capacity_in(source.comments.len(), allocator);
    for comment in &source.comments {
        if top_level_starts.contains(&comment.attached_to) {
            comments.push(*comment);
        }
    }
    compiled.comments = comments;

    // Codegen reads comment content from `source_text` via span offsets, so the
    // compiled program must point at the same source as the original.
    compiled.source_text = source.source_text;
}

/// Convenience wrapper — parses source text, runs semantic analysis, then transforms.
pub fn transform_source<'a>(
    source_text: &'a str,
    source_type: oxc_span::SourceType,
    allocator: &'a oxc_allocator::Allocator,
    options: PluginOptions,
) -> TransformResult<'a> {
    let parsed = oxc_parser::Parser::new(allocator, source_text, source_type).parse();
    transform(&parsed.program, allocator, options)
}

/// Lint a pre-parsed program — like [`transform`] but only collects diagnostics.
pub fn lint(program: &oxc_ast::ast::Program, options: PluginOptions) -> LintResult {
    let mut opts = options;
    opts.no_emit = true;

    // `no_emit` yields `program: None`; a local arena for the conversion suffices.
    let allocator = oxc_allocator::Allocator::default();
    let result = transform(program, &allocator, opts);
    LintResult { diagnostics: result.diagnostics }
}

/// Convenience wrapper — parses source text, runs semantic analysis, then lints.
pub fn lint_source(
    source_text: &str,
    source_type: oxc_span::SourceType,
    options: PluginOptions,
) -> LintResult {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, source_text, source_type).parse();
    lint(&parsed.program, options)
}

// End-to-end smoke tests: oxc parse + semantic -> convert -> compile -> convert
// back -> codegen.
#[cfg(test)]
mod tests {
    use react_compiler::entrypoint::plugin_options::PluginOptions;

    use super::transform_source;

    fn options() -> PluginOptions {
        // The upstream options type is constructed typed (it has no `Deserialize`);
        // only `filename` differs from the compiler's standard defaults.
        PluginOptions {
            filename: Some("Component.jsx".to_string()),
            ..super::default_plugin_options()
        }
    }

    #[test]
    fn memoizes_a_component_end_to_end() {
        let source = "function Component(props) {\n  \
            return <div onClick={() => props.onClick()}>{props.text}</div>;\n}\n";

        let allocator = oxc_allocator::Allocator::default();
        let result = transform_source(source, oxc_span::SourceType::tsx(), &allocator, options());

        assert!(!result.diagnostics.has_errors(), "unexpected errors: {:?}", result.diagnostics);
        assert!(
            !result.diagnostics.has_warnings(),
            "unexpected warnings: {:?}",
            result.diagnostics
        );
        let program = result.program.expect("React Compiler should have transformed the component");

        let output = oxc_codegen::Codegen::new().build(&program).code;

        assert!(
            output.contains("react/compiler-runtime"),
            "expected the compiler-runtime cache import in output:\n{output}"
        );
        assert!(
            output.contains("_c("),
            "expected memo cache reads (`_c(...)`) in output:\n{output}"
        );
    }

    #[test]
    fn skips_non_react_code() {
        let source = "function add(a, b) {\n  return a + b;\n}\n";
        let allocator = oxc_allocator::Allocator::default();
        let result = transform_source(source, oxc_span::SourceType::tsx(), &allocator, options());
        assert!(result.program.is_none(), "non-React code must not be transformed");
    }

    /// TypeScript-only constructs (`declare global`, `import =`, `export =`,
    /// overload signatures, `#field in obj`) round-trip without panicking while the
    /// component still compiles.
    #[test]
    fn typescript_only_constructs_round_trip() {
        let source = "\
import legacy = require('legacy');\n\
declare global {\n  interface Window { __APP__: number; }\n}\n\
declare function ambient(x: number): void;\n\
function overloaded(x: number): number;\n\
function overloaded(x: string): string;\n\
function overloaded(x: unknown): unknown { return x; }\n\
class Brand {\n  #brand = 1;\n  static isBrand(obj: object) { return #brand in obj; }\n}\n\
function Component(props) {\n  return <div>{props.text}</div>;\n}\n\
export = legacy;\n";

        let allocator = oxc_allocator::Allocator::default();
        let result = transform_source(source, oxc_span::SourceType::tsx(), &allocator, options());
        let program = result.program.unwrap_or_else(|| {
            panic!("component should be compiled; diagnostics: {:?}", result.diagnostics)
        });
        let output = oxc_codegen::Codegen::new().build(&program).code;
        assert!(
            output.contains("react/compiler-runtime"),
            "expected compiler-runtime import in output:\n{output}"
        );
        assert!(output.contains("declare global"), "`declare global` lost:\n{output}");
        assert!(output.contains("import legacy"), "`import =` lost:\n{output}");
        assert!(
            output.contains("function overloaded(x: number): number;"),
            "overload signature lost:\n{output}"
        );
        assert!(
            !output.contains("function overloaded(x: number): number {}"),
            "overload signature gained an empty body:\n{output}"
        );
        assert!(output.contains("export = legacy"), "`export =` lost:\n{output}");
    }

    /// TS wrappers in assignment-target / for-head positions must not crash the
    /// converter (the compiler may still decline to compile them).
    #[test]
    fn ts_wrapped_assignment_targets_do_not_panic() {
        let cases = [
            "function Component(props) {\n  let x = 0;\n  (x as number) = props.x;\n  return <div>{x}</div>;\n}\n",
            "function Component(props) {\n  let x = 0;\n  x! = props.x;\n  return <div>{x}</div>;\n}\n",
            "function Component(props) {\n  const o = props.o;\n  for ((o.k as string) in props.src) {}\n  return <div />;\n}\n",
            "function Component(props) {\n  const o = props.o;\n  for (o.k! of props.src) {}\n  return <div />;\n}\n",
            "function Component(props) {\n  let [a] = props.p;\n  ([a!] = props.q);\n  return <div>{a}</div>;\n}\n",
        ];
        let opts = options();
        for source in cases {
            let allocator = oxc_allocator::Allocator::default();
            let _ = transform_source(source, oxc_span::SourceType::tsx(), &allocator, opts.clone());
        }
    }

    /// Class bodies are stubbed by the converter and re-parsed from source on the
    /// way back, so members survive.
    #[test]
    fn class_body_is_preserved() {
        let source = "\
class Store {\n  count = 0;\n  increment() {\n    this.count++;\n  }\n}\n\
function Component(props) {\n  return <div>{props.text}</div>;\n}\n";
        let allocator = oxc_allocator::Allocator::default();
        let result = transform_source(source, oxc_span::SourceType::tsx(), &allocator, options());
        let program = result.program.expect("component should be compiled");
        let output = oxc_codegen::Codegen::new().build(&program).code;
        assert!(output.contains("react/compiler-runtime"), "component should memoize:\n{output}");
        assert!(output.contains("count = 0"), "class field lost:\n{output}");
        assert!(output.contains("increment("), "class method lost:\n{output}");
    }

    #[test]
    fn unsupported_sibling_ast_forms_are_preserved() {
        let source = "\
import './style.css';\n\
export * as ns from './mod';\n\
function helper() {\n\
  const C = class {\n\
    method() {\n\
      return 1;\n\
    }\n\
  };\n\
  return 123n + BigInt(new C().method());\n\
}\n\
function Component(props) {\n\
  return <div>{props.text}</div>;\n\
}\n";
        let allocator = oxc_allocator::Allocator::default();
        let result = transform_source(source, oxc_span::SourceType::tsx(), &allocator, options());
        let program = result.program.expect("component should be compiled");
        let output = oxc_codegen::Codegen::new().build(&program).code;

        assert!(output.contains("react/compiler-runtime"), "component should memoize:\n{output}");
        assert!(output.contains("import \"./style.css\";"), "bare import changed:\n{output}");
        assert!(
            !output.contains("import {} from \"./style.css\""),
            "bare import became an empty named import:\n{output}"
        );
        assert!(
            output.contains("export * as ns from \"./mod\";"),
            "namespace export lost:\n{output}"
        );
        assert!(output.contains("method() {"), "class expression method lost:\n{output}");
        assert!(output.contains("return 1;"), "class expression body lost:\n{output}");
        assert!(output.contains("123n"), "BigInt literal lost:\n{output}");
        assert!(!output.contains("123nn"), "BigInt literal gained an extra suffix:\n{output}");
    }

    #[test]
    fn typescript_surface_syntax_is_preserved_around_compiled_code() {
        let source = "\
import { createContext, forwardRef } from 'react';\n\
type Props = { text: string };\n\
declare const Generic: React.FC<Props>;\n\
declare const tag: <T>(strings: TemplateStringsArray) => string;\n\
class Box<T> {}\n\
const settings = { mode: 'dark' } as const;\n\
const Context = createContext<Props | undefined>(undefined);\n\
const typedValue: string = settings.mode as string;\n\
const checked = settings.mode satisfies string;\n\
const boxed = new Box<Props>();\n\
const tagged = tag<Props>`value`;\n\
const Wrapped = forwardRef<HTMLDivElement, Props>(({ text }: Props, ref): JSX.Element => {\n\
  const label: string = text satisfies string;\n\
  return <div ref={ref}>{label}</div>;\n\
});\n\
function renderGeneric(props: Props): JSX.Element {\n\
  'use no memo';\n\
  try {\n\
    return <Generic<Props> text={props.text} />;\n\
  } catch (error: unknown) {\n\
    return <Generic<Props> text={String(error)} />;\n\
  }\n\
}\n\
function Component(props: Props): JSX.Element {\n\
  return <Context.Provider value={{ text: props.text } as Props}>\n\
    <Wrapped text={props.text} />\n\
  </Context.Provider>;\n\
}\n";

        let allocator = oxc_allocator::Allocator::default();
        let result = transform_source(source, oxc_span::SourceType::tsx(), &allocator, options());
        let program = result.program.unwrap_or_else(|| {
            panic!("component should compile; diagnostics: {:?}", result.diagnostics)
        });
        let output = oxc_codegen::Codegen::new().build(&program).code;

        assert!(output.contains("react/compiler-runtime"), "component should memoize:\n{output}");
        assert!(output.contains("as const"), "`as const` lost:\n{output}");
        assert!(
            output.contains("declare const Generic: React.FC<Props>"),
            "declared variable type annotation lost:\n{output}"
        );
        assert!(
            output.contains("createContext<Props | undefined>"),
            "call type arguments lost:\n{output}"
        );
        assert!(output.contains("typedValue: string"), "variable type annotation lost:\n{output}");
        assert!(output.contains("as string"), "`as` expression lost:\n{output}");
        assert!(output.contains("satisfies string"), "`satisfies` expression lost:\n{output}");
        assert!(output.contains("new Box<Props>()"), "`new` type arguments lost:\n{output}");
        assert!(
            output.contains("tag<Props>`value`"),
            "tagged template type arguments lost:\n{output}"
        );
        assert!(
            output.contains("forwardRef<HTMLDivElement, Props>"),
            "generic forwardRef call lost type arguments:\n{output}"
        );
        assert!(output.contains(": Props"), "parameter type lost:\n{output}");
        assert!(output.contains(": JSX.Element"), "return type lost:\n{output}");
        assert!(output.contains("catch (error: unknown)"), "catch type lost:\n{output}");
        assert!(output.contains("<Generic<Props>"), "generic JSX type arguments lost:\n{output}");
    }

    #[test]
    fn type_query_casts_are_renamed_with_value_bindings() {
        let source = "\
type Field = { value?: string; optionsInputs?: Record<string, string> };\n\
function Component({ fields }: { fields: Field[] }) {\n\
  const field = { value: 'outer', optionsInputs: {} };\n\
  const nodes = fields.map((field, index) => {\n\
    const options = [{ value: field.value }];\n\
    const firstOptionInput = field.optionsInputs?.[options?.[0]?.value as keyof typeof field.optionsInputs];\n\
    return <div key={index}>{firstOptionInput}{field.value}</div>;\n\
  });\n\
  return <>{nodes}{field.value}</>;\n\
}\n";

        let allocator = oxc_allocator::Allocator::default();
        let result = transform_source(source, oxc_span::SourceType::tsx(), &allocator, options());
        let program = result.program.expect("component should be compiled");
        let output = oxc_codegen::Codegen::new().build(&program).code;

        assert!(output.contains("react/compiler-runtime"), "component should memoize:\n{output}");
        assert!(
            !output.contains("typeof field.optionsInputs"),
            "stale type query source was restored:\n{output}"
        );
        assert!(
            output.contains("typeof field_0.optionsInputs"),
            "type query binding was not renamed with the value binding:\n{output}"
        );
    }

    fn rename_collision_source_and_offset() -> (&'static str, usize) {
        let rename_source = "\
function makeResults(items, names) {\n\
  const results = [...items.map((x) => use(x.value)), ...names.map((x) => use(x))];\n\
  return results;\n\
}\n";
        let collision_offset = rename_source
            .find("use(x))")
            .map(|index| index + "use(".len())
            .expect("test source should contain the renamed reference");
        (rename_source, collision_offset)
    }

    fn assert_has_rename_collision_setup(output: &str) {
        assert!(
            output.contains("function _temp2(x_0)") || output.contains("function _temp(x_0)"),
            "test setup did not produce a compiler rename:\n{output}"
        );
    }

    #[test]
    fn source_extracted_class_spans_do_not_collide_with_rename_plan() {
        let (rename_source, collision_offset) = rename_collision_source_and_offset();

        let class_prefix = "export class C {\n  m() {\n    ";
        let declarator_prefix = "const [octokit, ";
        let padding_len = collision_offset
            .checked_sub(class_prefix.len() + declarator_prefix.len())
            .expect("class binding should be padded to the earlier rename position");
        let source = format!(
            "{rename_source}{class_prefix}{}{declarator_prefix}ghRepository] = foo();\n    return ghRepository.full_name;\n  }}\n}}\n",
            " ".repeat(padding_len)
        );

        let allocator = oxc_allocator::Allocator::default();
        let mut opts = options();
        opts.compilation_mode = "all".to_string();
        let result = transform_source(&source, oxc_span::SourceType::tsx(), &allocator, opts);
        let program = result.program.expect("file should be compiled");
        let output = oxc_codegen::Codegen::new().build(&program).code;

        assert_has_rename_collision_setup(&output);
        assert!(
            output.contains("const [octokit, ghRepository] = foo()"),
            "source-preserved class binding was renamed by an unrelated plan entry:\n{output}"
        );
        assert!(!output.contains("const [octokit, x_0]"), "class binding was corrupted:\n{output}");
    }

    #[test]
    fn jsx_attribute_string_entities_are_decoded() {
        let source = "\
function Component(props) {\n\
  return <TemplateLinkSection label={props.label} piiWarning='Use the iframe&apos;s payload.' />;\n\
}\n";

        let allocator = oxc_allocator::Allocator::default();
        let result = transform_source(source, oxc_span::SourceType::tsx(), &allocator, options());
        let program = result.program.expect("component should be compiled");
        let output = oxc_codegen::Codegen::new().build(&program).code;

        assert!(output.contains("react/compiler-runtime"), "component should memoize:\n{output}");
        assert!(
            output.contains("iframe's payload"),
            "JSX string attribute entity was not decoded:\n{output}"
        );
        assert!(
            !output.contains("iframe&apos;s payload"),
            "JSX string attribute entity leaked into runtime string:\n{output}"
        );
    }

    #[test]
    fn resource_management_declarations_bail_out() {
        let cases = [
            "\
function Component(props) {\n  using x = sideEffect();\n  return <div>{props.text}</div>;\n}\n",
            "\
async function Component(props) {\n  await using x = sideEffect();\n  return <div>{props.text}</div>;\n}\n",
        ];

        for source in cases {
            let allocator = oxc_allocator::Allocator::default();
            let result =
                transform_source(source, oxc_span::SourceType::tsx(), &allocator, options());

            assert!(result.program.is_none(), "resource management should skip React Compiler");
            assert!(
                result.diagnostics.is_empty(),
                "unexpected diagnostics: {:?}",
                result.diagnostics
            );
        }
    }

    #[test]
    fn exported_typescript_runtime_declarations_are_preserved() {
        let source = "\
export enum E { A }\n\
export namespace N {\n  export const value = E.A;\n}\n\
function Component(props) {\n  return <div>{E.A}{N.value}{props.text}</div>;\n}\n";
        let allocator = oxc_allocator::Allocator::default();
        let result = transform_source(source, oxc_span::SourceType::tsx(), &allocator, options());
        let program = result.program.expect("component should be compiled");
        let output = oxc_codegen::Codegen::new().build(&program).code;

        assert!(output.contains("export enum E"), "exported enum lost:\n{output}");
        assert!(output.contains("A"), "enum member lost:\n{output}");
        assert!(output.contains("export namespace N"), "exported namespace lost:\n{output}");
        assert!(output.contains("value = E.A"), "namespace body lost:\n{output}");
        assert!(
            !output.contains("declare const"),
            "exported TS declaration became a placeholder:\n{output}"
        );
    }

    /// A local `export { x }` that re-exports an imported binding must keep its
    /// `local` as an `IdentifierReference` after the round-trip, so semantic
    /// analysis links it to the import and downstream TypeScript import elision
    /// keeps the import alive instead of leaving a dangling export.
    #[test]
    fn local_reexport_keeps_its_import_binding() {
        use oxc_ast::ast::{ModuleExportName, Statement};

        let source = "\
import { Foo } from './foo';\n\
export { Foo };\n\
function Component(props) {\n  return <div>{props.text}</div>;\n}\n";
        let allocator = oxc_allocator::Allocator::default();
        let result = transform_source(source, oxc_span::SourceType::tsx(), &allocator, options());
        let program = result.program.expect("component should be compiled");

        let export = program
            .body
            .iter()
            .find_map(|stmt| match stmt {
                Statement::ExportNamedDeclaration(decl) if decl.source.is_none() => Some(decl),
                _ => None,
            })
            .expect("a local `export { Foo }` should round-trip");
        let local = &export.specifiers.first().expect("export specifier").local;
        assert!(
            matches!(local, ModuleExportName::IdentifierReference(_)),
            "local export `local` must be an IdentifierReference so semantic links it to the import",
        );

        // The freshly-built scoping for the compiled program must record the
        // export's reference to the import, or import elision would drop it.
        let semantic = oxc_semantic::SemanticBuilder::new().build(&program).semantic;
        let scoping = semantic.scoping();
        let foo = scoping
            .symbol_ids()
            .find(|&id| scoping.symbol_name(id) == "Foo")
            .expect("`Foo` import binding should exist");
        assert!(
            scoping.get_resolved_references(foo).next().is_some(),
            "the local re-export must reference the `Foo` import binding",
        );
    }

    /// A `React.memo(...)` component is anonymous; the prefilter must still see it.
    #[test]
    fn memo_wrapped_component_compiles() {
        let source = "React.memo((props) => {\n  return <div>{props.text}</div>;\n});\n";
        let allocator = oxc_allocator::Allocator::default();
        let result = transform_source(source, oxc_span::SourceType::tsx(), &allocator, options());
        let program = result.program.expect("memo-wrapped component should compile");
        let output = oxc_codegen::Codegen::new().build(&program).code;
        assert!(output.contains("react/compiler-runtime"), "should memoize:\n{output}");
        assert!(output.contains("_c("), "expected memo cache reads:\n{output}");
    }

    /// Diagnostics are surfaced at the compiler's own severity, not flattened.
    #[test]
    fn diagnostics_preserve_compiler_severity() {
        // A Rules of Hooks violation is an `Error`-severity diagnostic.
        let source = "function Component(props) {\n  if (props.cond) {\n    useState(0);\n  }\n  return <div>{props.text}</div>;\n}\n";
        let allocator = oxc_allocator::Allocator::default();
        let result = transform_source(source, oxc_span::SourceType::tsx(), &allocator, options());
        assert!(
            result.diagnostics.has_errors(),
            "Rules of Hooks violation should be reported as an error: {:?}",
            result.diagnostics
        );

        // A local named `fbt` is an unsupported-syntax bail-out — a warning, not an error.
        let source = "function Component() {\n  const fbt = \"span\";\n  return <fbt desc=\"label\">Hello</fbt>;\n}\n";
        let allocator = oxc_allocator::Allocator::default();
        let result = transform_source(source, oxc_span::SourceType::tsx(), &allocator, options());
        assert!(
            result.diagnostics.has_warnings(),
            "fbt bail-out should be reported as a warning: {:?}",
            result.diagnostics
        );
        assert!(
            !result.diagnostics.has_errors(),
            "fbt warning must not be reported as an error: {:?}",
            result.diagnostics
        );
    }

    /// Comments are dropped by the `react_compiler_ast` roundtrip, so
    /// `preserve_comments` carries top-level comments over from the original
    /// program. Comments inside a compiled function are not recovered.
    #[test]
    fn top_level_comments_are_preserved() {
        let source = "\
// keep: leading\n\
import { useState } from 'react';\n\
/** keep: jsdoc */\n\
function Component(props) {\n\
  // drop: inner\n\
  return <div onClick={() => props.onClick()}>{props.text}</div>;\n\
}\n\
// keep: trailing\n\
export default Component;\n";

        let allocator = oxc_allocator::Allocator::default();
        let result = transform_source(source, oxc_span::SourceType::tsx(), &allocator, options());
        let program = result.program.expect("component should be compiled");
        let output = oxc_codegen::Codegen::new().build(&program).code;

        assert!(output.contains("react/compiler-runtime"), "component should memoize:\n{output}");
        assert!(output.contains("// keep: leading"), "leading comment lost:\n{output}");
        assert!(output.contains("/** keep: jsdoc */"), "jsdoc comment lost:\n{output}");
        assert!(output.contains("// keep: trailing"), "trailing comment lost:\n{output}");
        assert!(
            !output.contains("// drop: inner"),
            "inner comment should not be recovered:\n{output}"
        );
    }
}
