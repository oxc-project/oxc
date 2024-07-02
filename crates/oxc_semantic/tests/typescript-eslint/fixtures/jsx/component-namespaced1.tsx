const X = {
  Foo() {},
};
const Foo = 1; // should be unreferenced

<X.Foo />;
