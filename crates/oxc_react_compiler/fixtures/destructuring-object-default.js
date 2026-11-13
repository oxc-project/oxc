function Component(props) {
  const {x: {y} = {y: 'default'}} = props.y;
  return y;
}

const sproutInputs = [
  [{y:{}}],
  [{y:{x:{y:42}}}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
