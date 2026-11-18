// Valid because the loop doesn't change the order of hooks calls.
function RegressionTest() {
  const res = [];
  const additionalCond = true;
  for (let i = 0; i !== 10 && additionalCond; ++i) {
    res.push(i);
  }
  React.useLayoutEffect(() => {});
}


const sproutInputs = [
  [],
  [],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return RegressionTest(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
