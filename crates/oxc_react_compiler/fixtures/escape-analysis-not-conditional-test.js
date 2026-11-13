function Component(props) {
  const x = [props.a];
  const y = x ? props.b : props.c;
  return y;
}

const sproutInputs = [
  [{"a":1,"b":2,"c":3}],
  [{"a":11,"b":12,"c":13}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
