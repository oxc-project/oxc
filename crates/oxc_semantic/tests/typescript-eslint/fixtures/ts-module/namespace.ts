namespace Foo {
  export const x = 1;
  Foo.x;
}

const unresolved = x;
Foo.x;
