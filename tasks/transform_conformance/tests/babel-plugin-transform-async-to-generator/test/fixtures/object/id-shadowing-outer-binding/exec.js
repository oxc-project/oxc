function customIteratorMethod() {
  let previous = null;
  let next = null;
  return {
    previous: async () =>
      previous || (previous = "previous value"),
    next: async () => next || (next = "next value"),
  };
}

return (async () => {
  const iter = customIteratorMethod();
  expect(await iter.next()).toBe("next value");
  expect(await iter.previous()).toBe("previous value");
  expect(await iter.next()).toBe("next value");
})();
