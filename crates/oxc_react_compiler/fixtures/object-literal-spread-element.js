function Component(props) {
  const x = {...props.foo};
  return x;
}

const sproutInputs = [
  [{foo:{a:1}}],
  [{foo:{a:2,b:3}}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
