function Component(props) {
  if (props.cond) {
    return undefined;
  }
  return props.value;
}

const sproutInputs = [
  [{"cond":false,"value":2}],
  [{"cond":true,"value":12}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
