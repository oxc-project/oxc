function* yieldKey() {
  return class {
    static [yield "a"] = 1;
  };
}

async function awaitKey(k: Promise<string>) {
  return class {
    static [await k] = 1;
  };
}

function thisKey(this: { k: string }) {
  return class {
    static [this.k] = 1;
  };
}

function argumentsKey(_k: string) {
  return class {
    [arguments[0]] = 1;
  };
}

function* afterLastComputedKey() {
  return class {
    [yield "a"] = 1;
    [k()]() {}
    static [yield "b"] = 2;
    m() {}
  };
}

function* afterHeritage() {
  return class extends base() {
    static [yield "a"] = 1;
  };
}

function* beforeDeclaration() {
  class C {
    static [yield "a"] = 1;
  }
  return C;
}

function* beforeNamedExpression() {
  const C = class {
    static [yield "a"] = 1;
  };
  return C;
}

function* uninitializedKey() {
  return class {
    static [yield "a"]: number;
    [yield "b"]: number;
    static erased: number;
    erased: number;
  };
}

class StaticBlock {
  static [k()] = 1;
}
