function Component(props) {
  useEffect(() => {
    const pathMap = new Map();
    const collectPaths = obj => {
      if (obj != null && typeof obj === 'object') {
        if (Array.isArray(obj)) {
          obj.forEach(item => collectPaths(item));
        } else {
          Object.values(obj).forEach(value => collectPaths(value));
        }
      }
    };
    collectPaths(props.data);
  }, [props.data]);
  return <div />;
}

import {useEffect} from 'react';
const sproutInputs = [
  [{data:{a:[1,{b:2}]}}],
  [{data:[null,{c:[3]}]}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
