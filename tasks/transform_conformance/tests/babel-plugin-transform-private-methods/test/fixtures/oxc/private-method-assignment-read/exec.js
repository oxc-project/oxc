let methodCalls = 0;

class Methods {
  #method() {
    methodCalls++;
  }

  static #staticMethod() {
    methodCalls++;
  }

  method() {
    return this.#method;
  }

  static method() {
    return this.#staticMethod;
  }

  run(operator, rhs, object = this) {
    switch (operator) {
      case "||=": return (object.#method ||= rhs());
      case "??=": return (object.#method ??= rhs());
      case "&&=": return (object.#method &&= rhs());
      case "+=": return (object.#method += rhs());
      case "-=": return (object.#method -= rhs());
      case "*=": return (object.#method *= rhs());
      case "/=": return (object.#method /= rhs());
      case "%=": return (object.#method %= rhs());
      case "**=": return (object.#method **= rhs());
      case "<<=": return (object.#method <<= rhs());
      case ">>=": return (object.#method >>= rhs());
      case ">>>=": return (object.#method >>>= rhs());
      case "&=": return (object.#method &= rhs());
      case "^=": return (object.#method ^= rhs());
      case "|=": return (object.#method |= rhs());
      case "pre++": return ++object.#method;
      case "post++": return object.#method++;
      case "pre--": return --object.#method;
      case "post--": return object.#method--;
    }
  }

  static run(operator, rhs, object = this) {
    switch (operator) {
      case "||=": return (object.#staticMethod ||= rhs());
      case "??=": return (object.#staticMethod ??= rhs());
      case "&&=": return (object.#staticMethod &&= rhs());
      case "+=": return (object.#staticMethod += rhs());
      case "-=": return (object.#staticMethod -= rhs());
      case "*=": return (object.#staticMethod *= rhs());
      case "/=": return (object.#staticMethod /= rhs());
      case "%=": return (object.#staticMethod %= rhs());
      case "**=": return (object.#staticMethod **= rhs());
      case "<<=": return (object.#staticMethod <<= rhs());
      case ">>=": return (object.#staticMethod >>= rhs());
      case ">>>=": return (object.#staticMethod >>>= rhs());
      case "&=": return (object.#staticMethod &= rhs());
      case "^=": return (object.#staticMethod ^= rhs());
      case "|=": return (object.#staticMethod |= rhs());
      case "pre++": return ++object.#staticMethod;
      case "post++": return object.#staticMethod++;
      case "pre--": return --object.#staticMethod;
      case "post--": return object.#staticMethod--;
    }
  }

  fromCall(operator, getObject, rhs) {
    switch (operator) {
      case "||=": return (getObject().#method ||= rhs());
      case "&&=": return (getObject().#method &&= rhs());
      case "+=": return (getObject().#method += rhs());
      case "post++": return getObject().#method++;
    }
  }

  static fromCall(operator, getObject, rhs) {
    switch (operator) {
      case "||=": return (getObject().#staticMethod ||= rhs());
      case "&&=": return (getObject().#staticMethod &&= rhs());
      case "+=": return (getObject().#staticMethod += rhs());
      case "post++": return getObject().#staticMethod++;
    }
  }
}

const binaryOperators = ["+=", "-=", "*=", "/=", "%=", "**=", "<<=", ">>=", ">>>=", "&=", "^=", "|="];
const updateOperators = ["pre++", "post++", "pre--", "post--"];

for (const receiver of [new Methods(), Methods]) {
  let rhsCalls = 0;
  const rhs = () => {
    rhsCalls++;
    return 2;
  };
  const method = receiver.method();

  for (const operator of ["||=", "??="]) {
    expect(receiver.run(operator, rhs)).toBe(method);
    expect(rhsCalls).toBe(0);
    expect(methodCalls).toBe(0);
  }

  for (const operator of ["&&=", ...binaryOperators]) {
    rhsCalls = 0;
    expect(() => receiver.run(operator, rhs)).toThrow(TypeError);
    expect(rhsCalls).toBe(1);
    expect(methodCalls).toBe(0);
  }

  for (const operator of updateOperators) {
    rhsCalls = 0;
    expect(() => receiver.run(operator, rhs)).toThrow(TypeError);
    expect(rhsCalls).toBe(0);
    expect(methodCalls).toBe(0);
  }

  // A failed brand check must precede RHS evaluation for every compound operator.
  for (const operator of ["||=", "??=", "&&=", ...binaryOperators, ...updateOperators]) {
    rhsCalls = 0;
    expect(() => receiver.run(operator, rhs, {})).toThrow(TypeError);
    expect(rhsCalls).toBe(0);
    expect(methodCalls).toBe(0);
  }

  let objectCalls = 0;
  const getObject = () => {
    objectCalls++;
    return receiver;
  };
  expect(receiver.fromCall("||=", getObject, rhs)).toBe(method);
  expect(objectCalls).toBe(1);
  expect(rhsCalls).toBe(0);
  expect(methodCalls).toBe(0);

  for (const operator of ["&&=", "+=", "post++"]) {
    objectCalls = 0;
    rhsCalls = 0;
    expect(() => receiver.fromCall(operator, getObject, rhs)).toThrow(TypeError);
    expect(objectCalls).toBe(1);
    expect(rhsCalls).toBe(operator === "post++" ? 0 : 1);
    expect(methodCalls).toBe(0);
  }
}
