export declare type MyType = string
export declare enum MyEnum {
  Foo,
  Bar,
  Baz
}
export declare interface Foo {
  native: string | number
  typedef: MyType
  enum: MyEnum
}

export declare abstract class Bar {
  abstract foo(): Foo

  method();
}

export declare function getFoo() : MyType;

export declare module MyModule {
  export function ModuleFunction();
}

export declare namespace MyNamespace {
  export function NamespaceFunction();

  export module NSModule {
    export function NSModuleFunction();
  }
}

interface NotExported {}
