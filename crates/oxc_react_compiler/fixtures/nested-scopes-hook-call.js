function component(props) {
  let x = [];
  let y = [];
  y.push(useHook(props.foo));
  x.push(y);
  return x;
}

import {useHook} from 'shared-runtime';
const sproutInputs = [
  [{foo:1}],
  [{foo:2}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
