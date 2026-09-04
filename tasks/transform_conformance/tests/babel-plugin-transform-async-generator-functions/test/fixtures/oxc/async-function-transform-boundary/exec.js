function outer() {
  return async () => {
    void this;
    return eval("typeof _this");
  };
}

class Base {
  get value() {
    return 1;
  }
}

class Derived extends Base {
  field = async () => {};

  async direct() {
    super.value;
    return eval("typeof _superprop_getValue");
  }

  arrow() {
    return async () => {
      super.value;
      return eval("typeof _superprop_getValue2");
    };
  }

  async *generator() {
    yield super.value;
  }
}

const instance = new Derived();
return Promise.all([outer()(), instance.direct(), instance.arrow()(), instance.generator().next()]).then(function (results) {
  expect(results[0]).toBe("undefined");
  expect(results[1]).toBe("undefined");
  expect(results[2]).toBe("undefined");
  expect(results[3].value).toBe(1);
  expect(instance.field.name).toBe("field");
});
