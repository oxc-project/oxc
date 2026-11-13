function foo(a, b) {
  if (a == null) {
    return null;
  } else {
    return b;
  }
}

const sproutInputs = [
  [null,1],
  [true,2],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return foo(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
