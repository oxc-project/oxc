function MyApp(props) {
  let res;
  if (props.cond) {
    return;
  } else {
    res = 1;
  }
}

const sproutInputs = [
  [{"cond":false}],
  [{"cond":true}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return MyApp(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
