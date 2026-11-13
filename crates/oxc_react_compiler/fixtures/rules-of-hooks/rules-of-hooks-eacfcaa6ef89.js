// Valid because hooks can be used in anonymous function arguments to
// memo.
const MemoizedFunction = memo(function (props) {
  useHook();
  return <button {...props} />;
});

import {memo} from 'react';
import {useHook} from 'shared-runtime';
const sproutInputs = [
  [],
  [],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return React.createElement(MemoizedFunction, args[0]);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
