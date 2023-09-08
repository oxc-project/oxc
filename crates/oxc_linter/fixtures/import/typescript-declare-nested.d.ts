declare namespace foo {
  interface SomeInterface {
    a: string;
  }
}

declare namespace foo.bar {
  interface SomeOtherInterface {
    b: string;
  }

  function MyFunction();
}

export = foo;
