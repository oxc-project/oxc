function Component({cond}) {
  let count = 0;
  let cb = () => count++;
  const run = async () => { await 0; cb(); };
  run(); cb = () => null;
  return null;
}


const sproutInputs = [
  [{"cond":true}],
  [{"cond":false}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
