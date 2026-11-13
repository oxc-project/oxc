function Component(props) {
  let x = [1, 2, 3];
  do {
    mutate(x);
    break;
  } while (props.cond);
  return x;
}

import {mutate} from 'shared-runtime';
const sproutInputs = [
  [{cond:true}],
  [{cond:false}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
