//! End-to-end integration tests: oxc parse + semantic -> compile -> codegen.

use oxc_allocator::Allocator;
use oxc_ast::ast::{ModuleExportName, Program, Statement};
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

use oxc_react_compiler::{PluginOptions, TransformResult, transform};

fn options() -> PluginOptions {
    PluginOptions::default()
}

/// Parse `source_text` then run the compiler in place, returning the
/// (possibly rewritten) program together with the result.
fn transform_source<'a>(
    source_text: &'a str,
    source_type: SourceType,
    allocator: &'a Allocator,
    options: PluginOptions,
) -> (Program<'a>, TransformResult<'a>) {
    let mut program = Parser::new(allocator, source_text, source_type).parse().program;
    let mut result = {
        let semantic = SemanticBuilder::new().with_build_nodes(true).build(&program).semantic;
        transform(&program, &semantic, allocator, options)
    };
    if let Some(compiled) = result.program.take() {
        program = compiled;
    }
    (program, result)
}

#[test]
fn memoizes_a_component_end_to_end() {
    let source = "function Component(props) {\n  \
        return <div onClick={() => props.onClick()}>{props.text}</div>;\n}\n";

    let allocator = Allocator::default();
    let (program, result) = transform_source(source, SourceType::tsx(), &allocator, options());

    assert!(!result.diagnostics.has_errors(), "unexpected errors: {:?}", result.diagnostics);
    assert!(!result.diagnostics.has_warnings(), "unexpected warnings: {:?}", result.diagnostics);
    assert!(result.changed, "React Compiler should have transformed the component");

    let output = Codegen::new().build(&program).code;

    assert!(
        output.contains("react/compiler-runtime"),
        "expected the compiler-runtime cache import in output:\n{output}"
    );
    assert!(output.contains("_c("), "expected memo cache reads (`_c(...)`) in output:\n{output}");
}

#[test]
fn skips_non_react_code() {
    let source = "function add(a, b) {\n  return a + b;\n}\n";
    let allocator = Allocator::default();
    let (_program, result) = transform_source(source, SourceType::tsx(), &allocator, options());
    assert!(!result.changed, "non-React code must not be transformed");
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

    let allocator = Allocator::default();
    let (program, result) = transform_source(source, SourceType::tsx(), &allocator, options());
    assert!(result.changed, "component should be compiled; diagnostics: {:?}", result.diagnostics);
    let output = Codegen::new().build(&program).code;
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
        let allocator = Allocator::default();
        let _ = transform_source(source, SourceType::tsx(), &allocator, opts.clone());
    }
}

/// Class bodies are stubbed by the converter and re-parsed from source on the
/// way back, so members survive.
#[test]
fn class_body_is_preserved() {
    let source = "\
class Store {\n  count = 0;\n  increment() {\n    this.count++;\n  }\n}\n\
function Component(props) {\n  return <div>{props.text}</div>;\n}\n";
    let allocator = Allocator::default();
    let (program, result) = transform_source(source, SourceType::tsx(), &allocator, options());
    assert!(result.changed, "component should be compiled");
    let output = Codegen::new().build(&program).code;
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
    let allocator = Allocator::default();
    let (program, result) = transform_source(source, SourceType::tsx(), &allocator, options());
    assert!(result.changed, "component should be compiled");
    let output = Codegen::new().build(&program).code;

    assert!(output.contains("react/compiler-runtime"), "component should memoize:\n{output}");
    assert!(output.contains("import \"./style.css\";"), "bare import changed:\n{output}");
    assert!(
        !output.contains("import {} from \"./style.css\""),
        "bare import became an empty named import:\n{output}"
    );
    assert!(output.contains("export * as ns from \"./mod\";"), "namespace export lost:\n{output}");
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

    let allocator = Allocator::default();
    let (program, result) = transform_source(source, SourceType::tsx(), &allocator, options());
    assert!(result.changed, "component should compile; diagnostics: {:?}", result.diagnostics);
    let output = Codegen::new().build(&program).code;

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
    assert!(output.contains("tag<Props>`value`"), "tagged template type arguments lost:\n{output}");
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

    let allocator = Allocator::default();
    let (program, result) = transform_source(source, SourceType::tsx(), &allocator, options());
    assert!(result.changed, "component should be compiled");
    let output = Codegen::new().build(&program).code;

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

#[test]
fn jsx_attribute_string_entities_are_decoded() {
    let source = "\
function Component(props) {\n\
  return <TemplateLinkSection label={props.label} piiWarning='Use the iframe&apos;s payload.' />;\n\
}\n";

    let allocator = Allocator::default();
    let (program, result) = transform_source(source, SourceType::tsx(), &allocator, options());
    assert!(result.changed, "component should be compiled");
    let output = Codegen::new().build(&program).code;

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
        let allocator = Allocator::default();
        let (_program, result) = transform_source(source, SourceType::tsx(), &allocator, options());

        assert!(!result.changed, "resource management should skip React Compiler");
        assert!(result.diagnostics.is_empty(), "unexpected diagnostics: {:?}", result.diagnostics);
    }
}

#[test]
fn exported_typescript_runtime_declarations_are_preserved() {
    let source = "\
export enum E { A }\n\
export namespace N {\n  export const value = E.A;\n}\n\
function Component(props) {\n  return <div>{E.A}{N.value}{props.text}</div>;\n}\n";
    let allocator = Allocator::default();
    let (program, result) = transform_source(source, SourceType::tsx(), &allocator, options());
    assert!(result.changed, "component should be compiled");
    let output = Codegen::new().build(&program).code;

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
    let source = "\
import { Foo } from './foo';\n\
export { Foo };\n\
function Component(props) {\n  return <div>{props.text}</div>;\n}\n";
    let allocator = Allocator::default();
    let (program, result) = transform_source(source, SourceType::tsx(), &allocator, options());
    assert!(result.changed, "component should be compiled");

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
    let semantic = SemanticBuilder::new().build(&program).semantic;
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
    let allocator = Allocator::default();
    let (program, result) = transform_source(source, SourceType::tsx(), &allocator, options());
    assert!(result.changed, "memo-wrapped component should compile");
    let output = Codegen::new().build(&program).code;
    assert!(output.contains("react/compiler-runtime"), "should memoize:\n{output}");
    assert!(output.contains("_c("), "expected memo cache reads:\n{output}");
}

/// Diagnostics are surfaced at the compiler's own severity, not flattened.
#[test]
fn diagnostics_preserve_compiler_severity() {
    // A Rules of Hooks violation is an `Error`-severity diagnostic.
    let source = "function Component(props) {\n  if (props.cond) {\n    useState(0);\n  }\n  return <div>{props.text}</div>;\n}\n";
    let allocator = Allocator::default();
    let (_program, result) = transform_source(source, SourceType::tsx(), &allocator, options());
    assert!(
        result.diagnostics.has_errors(),
        "Rules of Hooks violation should be reported as an error: {:?}",
        result.diagnostics
    );

    // A local named `fbt` is an unsupported-syntax bail-out — a warning, not an error.
    let source = "function Component() {\n  const fbt = \"span\";\n  return <fbt desc=\"label\">Hello</fbt>;\n}\n";
    let allocator = Allocator::default();
    let (_program, result) = transform_source(source, SourceType::tsx(), &allocator, options());
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

    let allocator = Allocator::default();
    let (program, result) = transform_source(source, SourceType::tsx(), &allocator, options());
    assert!(result.changed, "component should be compiled");
    let output = Codegen::new().build(&program).code;

    assert!(output.contains("react/compiler-runtime"), "component should memoize:\n{output}");
    assert!(output.contains("// keep: leading"), "leading comment lost:\n{output}");
    assert!(output.contains("/** keep: jsdoc */"), "jsdoc comment lost:\n{output}");
    assert!(output.contains("// keep: trailing"), "trailing comment lost:\n{output}");
    assert!(!output.contains("// drop: inner"), "inner comment should not be recovered:\n{output}");
}
