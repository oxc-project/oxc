const object = {
  async *values() {
    yield 1;
  },
};

class Values {
  async *values() {
    yield 2;
  }
}

return Promise.all([object.values().next(), new Values().values().next()]).then(function (results) {
  expect(results[0]).toEqual({ value: 1, done: false });
  expect(results[1]).toEqual({ value: 2, done: false });
});
