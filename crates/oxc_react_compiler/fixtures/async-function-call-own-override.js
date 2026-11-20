function Component({key, cond}) {
  let count = 0;
  const cb = () => count++;
  const run = async () => { await 0; cb(); };
  run.call = () => null;
  run.call(null);
  return null;
}


const sproutInputs = [
  [{"key":1,"cond":true}],
  [{"key":2,"cond":false}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
