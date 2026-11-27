function foo(a, b, c) {
  let x;
  if (a) {
    x = 2 - 1;
  } else {
    x = 0 + 1;
  }
  if (x === 1) {
    return b;
  } else {
    return c;
  }
}

const sproutInputs = [
  [true,2,3],
  [false,4,5],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return foo(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
