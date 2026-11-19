const object = {
  "\uD8001": async () => 42,
  "\uDC001": async () => 42,
  "12a": async () => 42,
  "12": async () => 42,
  "1 2": async () => 42,
  "1/2": async () => 42,
  404: async () => 42,
  "a- b": async () => 42,
  "-1a": async () => 42,
  "\u0661a": async () => 42,
};

return (async () => {
  expect(await object["\uD8001"]()).toBe(42);
  expect(await object["\uDC001"]()).toBe(42);
  expect(await object["12a"]()).toBe(42);
  expect(await object["12"]()).toBe(42);
  expect(await object["1 2"]()).toBe(42);
  expect(await object["1/2"]()).toBe(42);
  expect(await object[404]()).toBe(42);
  expect(await object["a- b"]()).toBe(42);
  expect(await object["-1a"]()).toBe(42);
  expect(await object["\u0661a"]()).toBe(42);
})();
