const object = {
  async *values() {
    void arguments;
    return eval("typeof _arguments");
  },
};

return object.values().next().then(function (result) {
  expect(result.value).toBe("undefined");
  expect(result.done).toBe(true);
});
