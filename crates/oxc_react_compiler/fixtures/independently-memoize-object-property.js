function foo(a, b, c) {
  const x = {a: a};
  // NOTE: this array should memoize independently from x, w only b,c as deps
  x.y = [b, c];

  return x;
}

const sproutInputs = [
  [1,2,3],
  [11,12,13],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return foo(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
