function f(a, b) {
  let x = []; // <- x starts being mutable here.
  if (a.length === 1) {
    if (b) {
      x.push(b); // <- x stops being mutable here.
    }
  }

  return <div>{x}</div>;
}

const sproutInputs = [
  [[1],2],
  [[],3],
  [[1],0],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return f(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[2], sproutInputs[2], sproutInputs[0]],
};
