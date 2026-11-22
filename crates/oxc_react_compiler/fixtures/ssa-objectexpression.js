function Component(props) {
  const a = 1;
  const b = 2;
  const x = {a: a, b: b};
  return x;
}

const sproutInputs = [
  [{}],
  [{}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
