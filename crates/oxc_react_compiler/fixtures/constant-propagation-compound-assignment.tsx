function Component({suffix, offset}: {suffix: string | null; offset: number | null}) {
  let text = 'prefix';
  let count = 10;

  if (suffix) {
    text += suffix;
  }
  if (offset) {
    count -= offset;
  }

  return <div>{text}: {count}</div>;
}


const sproutInputs = [
  [{suffix:null,offset:null}],
  [{suffix:"!",offset:3}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
