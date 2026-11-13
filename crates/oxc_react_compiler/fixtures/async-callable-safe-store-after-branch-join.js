function Component({cond}) {
  let count = 0;
  let cb = () => null;
  const safeAsync = async () => null;
  const asyncUsingCb = async () => { await 0; cb(); };
  let run;
  if (cond) { cb = () => count++; run = asyncUsingCb; }
  else { run = safeAsync; }
  cb = () => null;
  run();
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
