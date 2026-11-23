function Component(props) {
  return props?.items?.map?.(render)?.filter(Boolean) ?? [];
}


const sproutInputs = [
  [null],
  [{items:null}],
  [{items:[]}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
