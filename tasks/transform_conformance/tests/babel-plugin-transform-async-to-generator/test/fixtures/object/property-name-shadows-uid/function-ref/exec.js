const object = {
  _ref: async function (x) {
    return x;
  },
};

return (async () => {
  expect(await object._ref(42)).toBe(42);
})();
