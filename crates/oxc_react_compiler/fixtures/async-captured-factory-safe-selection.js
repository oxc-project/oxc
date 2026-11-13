function Component() {
  let count = 0;
  const mutate = () => count++;
  function make(safe, bad) { return safe; } function wrap(safe, bad) { return make(safe, bad); } wrap(async () => null, async () => { await 0; mutate(); })();
  return null;
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
