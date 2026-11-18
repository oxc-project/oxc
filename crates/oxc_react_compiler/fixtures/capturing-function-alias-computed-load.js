function bar(a) {
  let x = [a];
  let y = {};
  const f0 = function () {
    y = x[0];
  };
  f0();

  return y;
}

const sproutInputs = [
  [1],
  [11],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return bar(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
