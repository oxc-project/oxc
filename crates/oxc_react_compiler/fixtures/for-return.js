function Component(props) {
  for (let i = 0; i < props.count; i++) {
    return;
  }
}

const sproutInputs = [
  [{count:0}],
  [{count:3}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
