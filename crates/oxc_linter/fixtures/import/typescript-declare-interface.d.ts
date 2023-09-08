declare interface foo {
  a: string;
}

declare namespace SomeNamespace {
  type foobar = foo & {
    b: string;
  }
}

export = SomeNamespace
