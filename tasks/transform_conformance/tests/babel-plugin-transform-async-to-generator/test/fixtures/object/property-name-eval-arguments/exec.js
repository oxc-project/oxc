const arrows = {
  eval: async (x) => x,
  arguments: async (x) => x,
};

const functions = {
  eval: async function (x) {
    return x;
  },
  arguments: async function (x) {
    return x;
  },
};

return (async () => {
  expect(await arrows.eval(42)).toBe(42);
  expect(await arrows.arguments(42)).toBe(42);
  expect(await functions.eval(42)).toBe(42);
  expect(await functions.arguments(42)).toBe(42);
})();
