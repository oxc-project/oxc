const object = {
  async method() {
    void arguments;
    return eval("typeof _arguments");
  },
};

function outer() {
  return (async () => {
    void arguments;
    return eval("typeof _arguments2");
  })();
}

return Promise.all([object.method(), outer()]).then(function (results) {
  expect(results[0]).toBe("undefined");
  expect(results[1]).toBe("undefined");
});
