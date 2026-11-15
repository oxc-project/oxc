function Component(props) {
  let x = 0;
  for (let i = 0; i < props.count; ) {
    x += i;
    i += 1;
    continue;
  }
  return x;
}

const sproutInputs = [
  [{count:0}],
  [{count:4}],
  [{count:2}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[2], sproutInputs[2], sproutInputs[0]],
};
