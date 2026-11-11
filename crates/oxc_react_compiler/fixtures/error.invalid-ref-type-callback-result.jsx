import {useRef} from 'react';

export function Component() {
  const ref = useRef(null);
  const callbacks = {read: () => ({ref})};
  const {ref: alias} = callbacks.read();
  return <div>{alias.current}</div>;
}
