// @enableNewMutationAliasingModel:false
function Component() {
  const foo = () => {
    someGlobal = true;
  };
  // spreading a function is weird, but it doesn't call the function so this is allowed
  return <div {...foo} />;
}


const sproutInputs = [
  [],
  [],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
