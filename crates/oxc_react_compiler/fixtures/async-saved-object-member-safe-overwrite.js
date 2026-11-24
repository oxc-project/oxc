function Component() {
  let count = 0;
  const opts = {cb: () => count++};
  const run = async () => { const saved = opts; await 0; saved.cb(); }; run();
  opts.cb = () => null;
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
