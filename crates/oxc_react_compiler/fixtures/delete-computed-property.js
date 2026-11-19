function Component(props) {
  const x = {a: props.a, b: props.b};
  const key = 'b';
  delete x[key];
  return x;
}

const sproutInputs = [
  [{"a":1,"b":2}],
  [{"a":11,"b":12}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
