// Valid because the neither the condition nor the loop affect the hook call.
function App(props) {
  const someObject = {propA: true};
  for (const propName in someObject) {
    if (propName === true) {
    } else {
    }
  }
  const [myState, setMyState] = useState(null);
}

import {useState} from 'react';
const sproutInputs = [
  [{}],
  [{}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return App(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
