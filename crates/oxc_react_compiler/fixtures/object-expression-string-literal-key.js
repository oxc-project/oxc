function Component(props) {
  const x = {['foo']: props.foo};
  return x;
}

const sproutInputs = [
  [{"foo":1}],
  [{"foo":11}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
