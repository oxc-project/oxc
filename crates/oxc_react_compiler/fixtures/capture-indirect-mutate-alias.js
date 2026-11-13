function component(a) {
  let x = {a};
  const f0 = function () {
    let q = x;
    const f1 = function () {
      q.b = 1;
    };
    f1();
  };
  f0();

  return x;
}

const sproutInputs = [
  [1],
  [11],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
