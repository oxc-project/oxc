function component(a, b) {
  if (a > b) {
    let m = {};
  }
}

const sproutInputs = [
  [1,2],
  [11,12],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
