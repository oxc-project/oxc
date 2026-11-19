// @enableFunctionOutlining
// Minimized repro for C5: outlined function naming divergence.
// When source code has variables named _temp/_temp2, Babel's generateUid
// skips those names, but the Rust compiler's blind counter doesn't.
function Component(props) {
  const _temp = props.a;
  const _temp2 = props.b;
  return props.items.map(() => <div />);
}


const sproutInputs = [
  [{a:1,b:2,items:[1,2]}],
  [{a:3,b:4,items:[3]}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
