import lib = require('shared-runtime');

function useValue(value: number) {
  return lib.identity(value);
}


const sproutInputs = [
  [1],
  [2],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return useValue(args[0]);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
