function Component() {
  let count = 0;
  function make(...callbacks) { return async () => { await 0; callbacks[0](); }; }
  const args = [() => null, () => count++];
  const copied = [...args];
  const run = make(...copied); run();
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
