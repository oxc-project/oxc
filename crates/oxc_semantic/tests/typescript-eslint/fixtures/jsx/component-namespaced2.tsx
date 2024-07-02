const x = {
  Foo() {},
};
const Foo = 1; // should be unreferenced

<x.Foo />; // lower cased namespaces should still create a reference
