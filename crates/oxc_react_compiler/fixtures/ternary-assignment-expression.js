function ternary(props) {
  let x = 0;
  const y = props.a ? (x = 1) : (x = 2);
  return x + y;
}

const sproutInputs = [
  [{"a":1}],
  [{"a":11}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return ternary(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
