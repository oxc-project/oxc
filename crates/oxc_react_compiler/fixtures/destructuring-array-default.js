function Component(props) {
  const [[x] = ['default']] = props.y;
  return x;
}

const sproutInputs = [
  [{y:[]}],
  [{y:[[42]]}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
