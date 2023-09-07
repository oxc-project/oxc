export type MyType = string
export enum MyEnum {
  Foo,
  Bar,
  Baz
}
export interface Foo {
  native: string | number
  typedef: MyType
  enum: MyEnum
}

export abstract class Bar {
  abstract foo(): Foo

  method() {
    return "foo"
  }
}

export function getFoo() : MyType {
  return "foo"
}

export module MyModule {
  export function ModuleFunction() {}
}

export namespace MyNamespace {
  export function NamespaceFunction() {}

  export module NSModule {
    export function NSModuleFunction() {}
  }
}

interface NotExported {}
