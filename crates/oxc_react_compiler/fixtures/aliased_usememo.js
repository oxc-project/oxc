import {useMemo as myMemo} from 'react';

function Component({x}) {
  const v = myMemo(() => x * 2, [x]);
  return <div>{v}</div>;
}


const sproutInputs = [
  [{"x":1}],
  [{"x":2}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
