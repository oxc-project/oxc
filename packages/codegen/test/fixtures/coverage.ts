declare const value: boolean;
declare const tiny: boolean;

if (value)
  if (tiny) debugger;
  else debugger;
else if (tiny) debugger;
else debugger;

do {} while (false);
do; while (false);
do debugger; while (false);

switch (value) {}
switch (value) {
  case true:
    debugger;
    break;
  default:
    debugger;
}

try {
  debugger;
} catch {
  debugger;
} finally {
  debugger;
}

enum TemplateKeys {
  [`key`] = 1,
}

type PrefixNullable = ?number;
type PrefixNonNullable = !string;
type TupleForms = [required: string, optional?: number, ...rest: boolean[]];

type Source = { a: string };
type MappedPlus = { +readonly [K in keyof Source]+?: Source[K] };
type MappedMinus = { -readonly [K in keyof Source]-?: Source[K] };
type Imported = import("mod").Foo<Bar>;
type ImportedOptions = import("mod", { with: { type: "json" } }).Foo;
type Queried = typeof import("mod").Foo;
type IntrinsicArray = (intrinsic)[];
type IntrinsicIndex = (intrinsic)["x"];
type IntrinsicUnion = (intrinsic | string);
type IntrinsicConditional<T> = (intrinsic extends T ? string : number);

declare module "foo";
declare module "bar" {}
export {};
declare global {}
namespace N {
  interface I {}
}

type Keyed = {
  key: string;
  "quoted": number;
  [computed]: boolean;
  get value(): string;
  set value(value: string);
  (arg: number): string;
  new (arg: string): Date;
  readonly [index: string]: number;
};

interface Empty {}
interface Full<in T, out U> extends A<T>, B {
  readonly foo?: string;
  [key: string]: number;
  get value(): string;
  set value(value: string);
  <X>(value: X): X;
  new (value: string): Date;
}

const enum EmptyEnum {}
declare enum DeclaredEnum {
  A = 1,
  "B" = 2,
}
