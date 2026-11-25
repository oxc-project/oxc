function f(a) {
  let x;
  (() => {
    x = {};
  })();
  // this is not reactive on `x` as `x` is never reactive
  return <div x={x} />;
}

const sproutInputs = [
  [1],
  [11],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return f(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
