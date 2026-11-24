import {useRef} from 'react';

export function Component() {
  const ref = useRef(null);
  const empty = () => null;
  if (ref.current === empty()) {
    ref.current = 1;
  }
  return <div />;
}


const sproutInputs = [
  [],
  [],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
