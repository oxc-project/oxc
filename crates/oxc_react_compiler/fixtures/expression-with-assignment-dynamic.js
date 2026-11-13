function f(y) {
  let x = y;
  return x + (x = 2) + x;
}

const sproutInputs = [
  [1],
  [5],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return f(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
