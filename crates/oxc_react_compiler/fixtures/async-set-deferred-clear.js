function Component() {
  let count = 0;
  const cb = () => count++;
  const callbacks = new Set([cb]); const run = async () => { await 0; callbacks.forEach(fn => fn()); }; run(); callbacks.clear();
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
