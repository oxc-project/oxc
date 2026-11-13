function t(props) {
  let [foo, bar, ,] = props;
  return foo;
}

const sproutInputs = [
  [[1,2,3]],
  [[4,5,6]],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return t(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
