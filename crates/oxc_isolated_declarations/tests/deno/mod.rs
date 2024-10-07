//! Copy from <https://github.com/denoland/deno_graph/blob/main/src/fast_check/transform_dts.rs#L932-#L1532>
//! Make some changes to conform to the Isolated Declarations output

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_codegen::CodeGenerator;
    use oxc_isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions};
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    fn transform_dts_test(source: &str, expected: &str) {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path("test.ts").unwrap();
        let ret = Parser::new(&allocator, source, source_type).parse();
        let ret = IsolatedDeclarations::new(
            &allocator,
            source,
            &ret.trivias,
            IsolatedDeclarationsOptions { strip_internal: true },
        )
        .build(&ret.program);
        let actual = CodeGenerator::new().build(&ret.program).code;
        let expected_program = Parser::new(&allocator, expected, source_type).parse().program;
        let expected = CodeGenerator::new().build(&expected_program).code;
        assert_eq!(actual.trim(), expected.trim());
    }

    #[test]
    fn dts_function_test() {
        transform_dts_test(
            "export function foo(a: number): number {
  return {};
}",
            "export declare function foo(a: number): number;",
        );
        transform_dts_test(
            "export function foo(a: string): number;
export function foo(a: any): number {
  return {};
}",
            "export declare function foo(a: string): number;",
        );
        transform_dts_test(
            "export function foo(a = 2): number {
  return 2;
}",
            "export declare function foo(a?: number): number;",
        );
        transform_dts_test(
            "export function foo(a: string = 2): number {
  return 2;
}",
            "export declare function foo(a?: string): number;",
        );
    }

    #[test]
    fn dts_class_decl_test() {
        transform_dts_test(
            "export class Foo {
  a: number = 2;
  static b: number = 1;
  #b: number = 3;
  constructor(value: string) {
  return 42;
  }
  foo(): string {
  return \"abc\";
  }
  #bar(): number {
  return 2
  }
  get asdf(): number {

  }
  set asdf(value: number) {

  }

  static {

  }
}",
            "export declare class Foo {
  #private;
  a: number;
  static b: number;
  constructor(value: string);
  foo(): string;
  get asdf(): number;
  set asdf(value: number);
}",
        );
    }

    #[test]
    fn dts_class_decl_rest_test() {
        transform_dts_test(
            "export class Foo {
  constructor(...args: string[]) {}
}",
            "export declare class Foo {
  constructor(...args: string[]);
}",
        );
    }

    #[test]
    fn dts_class_decl_overloads_test() {
        transform_dts_test(
            "export class Foo {
  constructor(arg: string);
  constructor(arg: number);
  constructor(arg: any) {}
}",
            "export declare class Foo {
  constructor(arg: string);
  constructor(arg: number);
}",
        );

        transform_dts_test(
            "export class Foo {
  foo(arg: string);
  foo(arg: number);
  foo(arg: any) {}
}",
            "export declare class Foo {
  foo(arg: string);
  foo(arg: number);
}",
        );

        transform_dts_test(
            "export class Foo {
  constructor(arg: string);
  constructor(arg: number);
  constructor(arg: any) {}

  bar(arg: number): number {
  return 2
  }

  foo(arg: string);
  foo(arg: number);
  foo(arg: any) {}
}",
            "export declare class Foo {
  constructor(arg: string);
  constructor(arg: number);
  bar(arg: number): number;
  foo(arg: string);
  foo(arg: number);
}",
        );
    }

    #[test]
    fn dts_class_decl_prop_test() {
        transform_dts_test(
            "export class Foo { declare a: string }",
            "export declare class Foo {
  a: string;
}",
        );
    }

    #[test]
    fn dts_class_decl_prop_infer_test() {
        transform_dts_test(
            "export class Foo { foo = (a: string): string => ({} as any) }",
            "export declare class Foo {
  foo: (a: string) => string;
}",
        );
        transform_dts_test(
            "export class Foo { foo = function(a: string): void {} }",
            "export declare class Foo {
  foo: (a: string) => void;
}",
        );
    }

    #[test]
    fn dts_var_decl_test() {
        transform_dts_test("export const foo: number = 42;", "export declare const foo: number;");

        transform_dts_test("export var foo: number = 42;", "export declare var foo: number;");

        transform_dts_test("export let foo: number = 42;", "export declare let foo: number;");
    }

    #[test]
    fn dts_global_declare() {
        transform_dts_test(
            "declare global {
  interface String {
  fancyFormat(opts: StringFormatOptions): string;
  }
}",
            "declare global {
  interface String {
  fancyFormat(opts: StringFormatOptions): string;
  }
}",
        );
    }

    #[test]
    fn dts_inference() {
        transform_dts_test(
            "export const foo = null as string as number;",
            "export declare const foo: number;",
        );
    }

    #[test]
    fn dts_as_const() {
        transform_dts_test(
            "export const foo = [1, 2] as const;",
            "export declare const foo: readonly [1, 2];",
        );
        transform_dts_test(
            "export const foo = [1, ,2] as const;",
            "export declare const foo: readonly [1, undefined, 2];",
        );

        transform_dts_test(
            "export const foo = { str: \"bar\", bool: true, bool2: false, num: 42,   nullish: null } as const;",
            "export declare const foo: {
  readonly str: \"bar\";
  readonly bool: true;
  readonly bool2: false;
  readonly num: 42;
  readonly nullish: null;
};",
        );

        transform_dts_test(
            "export const foo = { str: [1, 2] as const } as const;",
            "export declare const foo: {
  readonly str: readonly [1, 2];
};",
        );
    }

    #[test]
    fn dts_literal_inference_ann() {
        transform_dts_test(
            "export const foo: number = \"abc\";",
            "export declare const foo: number;",
        );
        transform_dts_test("export let foo: number = \"abc\";", "export declare let foo: number;");
        transform_dts_test("export var foo: number = \"abc\";", "export declare var foo: number;");
    }

    #[test]
    fn dts_literal_inference() {
        transform_dts_test("export const foo = 42;", "export declare const foo = 42;");
        transform_dts_test("export const foo = \"foo\";", "export declare const foo = \"foo\";");
        transform_dts_test("export const foo = true;", "export declare const foo: boolean;");
        transform_dts_test("export const foo = false;", "export declare const foo: boolean;");
        transform_dts_test("export const foo = null;", "export declare const foo: null;");
        transform_dts_test("export let foo = undefined;", "export declare let foo: undefined;");
        transform_dts_test("export let foo = 10n;", "export declare let foo: bigint;");
    }

    #[test]
    fn dts_fn_expr() {
        transform_dts_test(
            "export let foo = function add(a: number, b: number): number {
  return a + b;
}",
            "export declare let foo: (a: number, b: number) => number;",
        );
        transform_dts_test(
            "export let foo = function add<T>([a, b]: T): void {}",
            "export declare let foo: <T>([a, b]: T) => void;",
        );
        transform_dts_test(
            "export let foo = function add<T>({a, b}: T): void {}",
            "export declare let foo: <T>({ a, b }: T) => void;",
        );
        transform_dts_test(
            "export let foo = function add(a = 2): void {}",
            "export declare let foo: (a?: number) => void;",
        );
        transform_dts_test(
            "export let foo = function add(...params: any[]): void {}",
            "export declare let foo: (...params: any[]) => void;",
        );
    }

    #[test]
    fn dts_fn_arrow_expr() {
        transform_dts_test(
            "export let foo = (a: number, b: number): number => {
  return a + b;
}",
            "export declare let foo: (a: number, b: number) => number;",
        );
        transform_dts_test(
            "export let foo = <T>([a, b]: T): void => {}",
            "export declare let foo: <T>([a, b]: T) => void;",
        );
        transform_dts_test(
            "export let foo = <T>({a, b}: T): void => {}",
            "export declare let foo: <T>({ a, b }: T) => void;",
        );
        transform_dts_test(
            "export let foo = (a = 2): void => {}",
            "export declare let foo: (a?: number) => void;",
        );

        transform_dts_test(
            "export let foo = (...params: any[]): void => {}",
            "export declare let foo: (...params: any[]) => void;",
        );
    }

    #[test]
    fn dts_type_export() {
        transform_dts_test("interface Foo {}", "interface Foo {\n}");
        transform_dts_test("type Foo = number;", "type Foo = number;");

        transform_dts_test("export interface Foo {}", "export interface Foo {\n}");
        transform_dts_test("export type Foo = number;", "export type Foo = number;");
    }

    #[test]
    fn dts_enum_export() {
        transform_dts_test(
            "export enum Foo { A, B }",
            "export declare enum Foo {\n  A=0,\n  B=1\n}",
        );
        transform_dts_test(
            "export const enum Foo { A, B }",
            "export declare const enum Foo {\n  A=0,\n  B=1\n}",
        );

        transform_dts_test(
            "export enum Foo { A = \"foo\", B = \"bar\" }",
            "export declare enum Foo {\n  A = \"foo\",\n  B = \"bar\"\n}",
        );

        // TODO: Enum rules https://www.typescriptlang.org/docs/handbook/enums.html
    }

    #[test]
    fn dts_default_export() {
        transform_dts_test(
            "export default function(a: number, b: number): number {};",
            "export default function(a: number, b: number): number;",
        );
        transform_dts_test(
            "export default function(a: number, b: number): number;
        export default function(a: number, b: number): any {
          return foo
        };",
            "export default function(a: number, b: number): number;",
        );
        transform_dts_test(
            "export default class {foo = 2};",
            "export default class {\n  foo: number;\n}",
        );
        transform_dts_test(
            "export default 42;",
            "declare const _default: number;\nexport default _default;",
        );
        transform_dts_test(
            "const a: number = 42; export default a;",
            "declare const a: number;\nexport default a;",
        );
    }

    #[test]
    fn dts_default_export_named() {
        transform_dts_test(
            "export { foo, bar } from \"foo\";",
            "export { foo, bar } from \"foo\";",
        );
    }

    #[test]
    fn dts_default_export_all() {
        transform_dts_test("export * as foo from \"foo\";", "export * as foo from \"foo\";");
    }
}
