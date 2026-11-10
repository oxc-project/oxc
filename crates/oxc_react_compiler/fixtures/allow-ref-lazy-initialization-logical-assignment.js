// @validateRefAccessDuringRender

import {useRef} from 'react';

// `ref.current ??=` is the same lazy initialization as
// `if (ref.current == null) ref.current = ...`.
function Component(props) {
  const ref = useRef(null);
  ref.current ??= props.value;
  return <Child ref={ref} />;
}

function Child({ref}) {
  'use no memo';
  return ref.current;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 42}],
};
