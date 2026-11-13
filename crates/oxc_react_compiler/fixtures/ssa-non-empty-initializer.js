function foo(a, b) {
  let x = [];
  if (a) {
    x = 1;
  }

  let y = x;
  return y;
}

const sproutInputs = [
  [true,1],
  [false,2],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return foo(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
