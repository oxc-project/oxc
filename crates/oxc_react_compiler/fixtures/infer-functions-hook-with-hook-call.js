// @compilationMode:"infer"
function useStateValue(props) {
  const [state, _] = useState(null);
  return [state];
}

import {useState} from 'react';
const sproutInputs = [
  [{}],
  [{}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return useStateValue(args[0]);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
