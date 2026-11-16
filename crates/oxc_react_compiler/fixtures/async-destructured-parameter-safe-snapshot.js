function Component() {
  let count = 0;
  const arg = {cb: () => null}; const run = async ({cb}) => { await 0; cb(); }; run(arg); arg.cb = () => count++;
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
