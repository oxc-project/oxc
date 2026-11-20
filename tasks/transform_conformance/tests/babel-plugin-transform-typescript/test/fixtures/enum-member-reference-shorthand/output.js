var Foo = function(Foo) {
  Foo[Foo["a"] = Math.random()] = "a";
  Foo[Foo["b"] = { a: Foo.a }.a] = "b";
  Foo[Foo["__proto__"] = Math.random()] = "__proto__";
  Foo[Foo["c"] = { ["__proto__"]: Foo.__proto__ }.__proto__] = "c";
  return Foo;
}(Foo || {});
