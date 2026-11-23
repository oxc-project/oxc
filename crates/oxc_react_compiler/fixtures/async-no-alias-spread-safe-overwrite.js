function Component({cond}) {
  let count = 0;
  let bad = () => count++;
  const callbacks = [async () => { await 0; bad(); }];
  [0].filter(...callbacks); bad = () => null;
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
