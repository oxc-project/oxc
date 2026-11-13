function Component(props) {
  debugger;
  if (props.cond) {
    debugger;
  } else {
    while (props.cond) {
      debugger;
    }
  }
  debugger;
}

const sproutInputs = [
  [{"cond":false}],
  [{"cond":true}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
