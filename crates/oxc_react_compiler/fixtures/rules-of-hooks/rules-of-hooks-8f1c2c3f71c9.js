// @expectNothingCompiled @compilationMode:"infer"
// Valid because components can use hooks.
function createComponentWithHook() {
  return function ComponentWithHook() {
    useHook();
  };
}

import {useHook} from 'shared-runtime';
const sproutInputs = [
  [],
  [],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return createComponentWithHook()();
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
