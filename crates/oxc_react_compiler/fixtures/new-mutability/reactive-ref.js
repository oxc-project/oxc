// @enableNewMutationAliasingModel
function ReactiveRefInEffect(props) {
  const ref1 = useRef('initial value');
  const ref2 = useRef('initial value');
  let ref;
  if (props.foo) {
    ref = ref1;
  } else {
    ref = ref2;
  }
  useEffect(() => print(ref));
}

import {useRef, useEffect} from 'react';
import {print} from 'shared-runtime';
const sproutInputs = [
  [{foo:1}],
  [{foo:2}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return ReactiveRefInEffect(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
