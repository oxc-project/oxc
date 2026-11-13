export function Component(recursive) {
  return function recursive(x) {
    const read = () => recursive;
    if (x) {
      return read;
    }
    recursive = null;
    return read;
  };
}


const sproutInputs = [
  [true],
  [false],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(null)(...args)();
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
