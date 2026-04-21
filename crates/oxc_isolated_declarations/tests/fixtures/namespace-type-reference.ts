// This ends up being `any` 
export type A = Foo.Bar;

namespace Foo {
  export type Bar = "bar"
}
