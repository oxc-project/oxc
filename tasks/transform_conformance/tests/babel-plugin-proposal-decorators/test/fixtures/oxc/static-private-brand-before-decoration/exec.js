const events = [];
function invoke(value) {
  events.push(value());
  return value;
}
function replace(value) {
  return function () {
    return value.call(this) + 10;
  };
}
class Example {
  static #before() { return 1; }
  @replace static #decorated() { return 2; }
  @invoke static read() {
    return Example.#before() + Example.#decorated() + Example.#after;
  }
  static get #after() { return 3; }
  static #field = 4;
  @invoke static fieldHasNoBrandYet() {
    return #field in Example;
  }
}
expect(events).toEqual([16, false]);
expect(Example.read()).toBe(16);

const Expression = class Inner {
  static #method() { return 5; }
  @invoke static read() { return Inner.#method(); }
};
expect(Expression.read()).toBe(5);
expect(events).toEqual([16, false, 5]);
