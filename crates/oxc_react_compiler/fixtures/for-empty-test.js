function Component(props) {
  let index = 0;
  let total = 0;

  for (;;) {
    index++;
    if (index < props.start) {
      continue;
    }
    total += index;
    if (index >= props.end) {
      break;
    }
  }

  return total;
}


const sproutInputs = [
  [{start:1,end:3}],
  [{start:3,end:5}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
