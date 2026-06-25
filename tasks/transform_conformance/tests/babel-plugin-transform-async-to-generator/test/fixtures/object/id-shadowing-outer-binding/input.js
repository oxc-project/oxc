function customIteratorMethod() {
  let previous = null;
  let next = null;
  return {
    previous: async () =>
      previous || (previous = "previous value"),
    next: async () => next || (next = "next value"),
  };
}
