class C {
  #fn;

  constructor(fn) {
    this.#fn = fn;
  }

  run(argument) {
    return (this?.#fn)?.(argument());
  }

  runNonNull(argument) {
    return (this?.#fn!)?.(argument());
  }

  runReceiver(o: C | undefined, argument) {
    return (o?.#fn!)?.(argument());
  }

  runRequired() {
    return (this?.#fn!)();
  }
}

let argumentCalls = 0;
const argument = () => ++argumentCalls;
const missing = new C(undefined);

expect(missing.run(argument)).toBeUndefined();
expect(missing.runNonNull(argument)).toBeUndefined();
expect(missing.runReceiver(missing, argument)).toBeUndefined();
expect(missing.run.call(undefined, argument)).toBeUndefined();
expect(missing.runNonNull.call(undefined, argument)).toBeUndefined();
expect(missing.runReceiver(undefined, argument)).toBeUndefined();
expect(argumentCalls).toBe(0);
expect(() => missing.runRequired()).toThrow(TypeError);

const receivers = [];
const argumentsSeen = [];
const callable = new C(function (value) {
  receivers.push(this);
  argumentsSeen.push(value);
  return this;
});

expect(callable.run(argument)).toBe(callable);
expect(callable.runNonNull(argument)).toBe(callable);
expect(missing.runReceiver(callable, argument)).toBe(callable);
expect(receivers).toEqual([callable, callable, callable]);
expect(argumentsSeen).toEqual([1, 2, 3]);
expect(argumentCalls).toBe(3);
