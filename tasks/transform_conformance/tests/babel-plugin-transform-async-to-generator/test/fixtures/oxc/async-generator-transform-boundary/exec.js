class Base {
  get value() {
    return 1;
  }
}

class Derived extends Base {
  async *values() {
    super.value;
    yield eval("typeof _superprop_getValue");
  }
}

return new Derived().values().next().then(function (result) {
  expect(result.value).toBe("undefined");
});
