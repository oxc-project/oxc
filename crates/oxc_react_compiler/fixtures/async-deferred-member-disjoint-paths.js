function Component({cond}) {
  let count = 0;
  const opts = {cb: () => null};
  const run = async () => { await 0; opts.cb(); };
  if (cond) { run(); } else { opts.cb = () => count++; }
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
