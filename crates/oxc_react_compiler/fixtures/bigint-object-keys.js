function component(value) {
  return {
    1n: value,
    0x10n: value,
    0o10n: value,
    0b10n: value,
    9007199254740993n: value,
  };
}


const sproutInputs = [
  [1],
  [2],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
