import {useRef} from 'react';

export function Component() {
  const ref = useRef(null);
  const empty = () => null;
  if (ref.current === empty()) {
    ref.current = 1;
  }
  return <div />;
}
