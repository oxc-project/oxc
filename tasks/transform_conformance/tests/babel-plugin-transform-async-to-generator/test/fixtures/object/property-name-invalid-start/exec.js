const object = {
  "\uD8001": async () => 42,
  "\uDC001": async () => 42,
};

return (async () => {
  expect(await object["\uD8001"]()).toBe(42);
  expect(await object["\uDC001"]()).toBe(42);
})();
