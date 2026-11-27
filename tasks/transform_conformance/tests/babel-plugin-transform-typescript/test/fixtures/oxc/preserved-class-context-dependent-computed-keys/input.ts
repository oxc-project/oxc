function* autoAccessor() {
  return class {
    static [yield "a"] = 1;
    accessor x = 0;
    [arguments[0]] = 2;
  };
}

function* decorated() {
  return @dec class {
    static [yield "a"] = 1;
  };
}

function* decoratedStaticMethod() {
  return class {
    @dec static m() {}
    static [yield "a"] = 1;
  };
}
