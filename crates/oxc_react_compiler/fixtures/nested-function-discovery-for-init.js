// @compilationMode:"annotation"
for (
  var useFoo = function useFoo() {
    'use memo';
    return [1, 2, 3];
  };
  false;

) {}


const sproutInputs = [
  [],
  [],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return useFoo();
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
