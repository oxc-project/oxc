export = AssignedNamespace;

declare namespace AssignedNamespace {
  type MyType = string
  enum MyEnum {
    Foo,
    Bar,
    Baz
  }

  interface Foo {
    native: string | number
    typedef: MyType
    enum: MyEnum
  }

  abstract class Bar {
    abstract foo(): Foo

    method();
  }

  export function getFoo() : MyType;

  export module MyModule {
    export function ModuleFunction();
  }

  export namespace MyNamespace {
    export function NamespaceFunction();

    export module NSModule {
      export function NSModuleFunction();
    }
  }

  // Export-assignment exports all members in the namespace, explicitly exported or not.
  // interface NotExported {}
}
