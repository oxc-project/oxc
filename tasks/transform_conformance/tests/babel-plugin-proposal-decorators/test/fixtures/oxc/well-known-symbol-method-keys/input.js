// A well-known symbol key can't collide with a string name, so methods keep their keys.
class A {
  @dec m() {}
  plain() {}
  *[Symbol.iterator]() {}
}

// The same well-known symbol twice can collide.
class B {
  @dec *[Symbol.iterator]() {}
  *[Symbol.iterator]() {}
}

// A shadowed `Symbol` can produce any key.
{
  const Symbol = { iterator: "m" };
  class C {
    @dec m() {}
    [Symbol.iterator]() {}
  }
}

// A decorated well-known symbol key keeps every key too.
class D {
  @dec *[Symbol.iterator]() {}
  m() {}
}

// Other properties of `Symbol` can hold any key.
class E {
  @dec m() {}
  [Symbol.foo]() {}
}

// A later well-known symbol can be missing in an ES2015 engine, making the key "undefined".
class F {
  @dec undefined() {}
  [Symbol.asyncIterator]() {}
}
