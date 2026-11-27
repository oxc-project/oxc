const object = {
  _ref: async (x) => x,
};

return (async () => {
  expect(await object._ref(42)).toBe(42);
})();
