function Component() {
  let count = 0;
  const mutate = () => count++;
  const tag = (strings, safe) => safe; const run = tag`${async () => null}${async () => { await 0; mutate(); }}`; run();
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
