// useMemo imported from 'react' should be detected and dropped.
// The module name matching must handle 'react' correctly.

import {useMemo} from 'react';

function Component({items}) {
  const sorted = useMemo(() => [...items].sort(), [items]);
  return <div>{sorted}</div>;
}

export default Component;


const sproutInputs = [
  [{items:[3,1,2]}],
  [{items:[5,4]}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
