// A typed `this` counts as a parameter, so `it(name, fn, timeout)` with `this` + `done` is not a test call
it("does something long enough to reach the width", function (this: Mocha.Context, done) {
  done();
}, 2000);

// A lone `this` keeps the test-call layout
it("does something long enough to reach the width", function (this: Mocha.Context) {
  return this.skip();
}, 2000);
