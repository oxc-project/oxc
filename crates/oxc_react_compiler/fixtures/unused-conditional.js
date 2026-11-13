function Component(props) {
  let x = 0;
  (x = 1) && (x = 2);
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
