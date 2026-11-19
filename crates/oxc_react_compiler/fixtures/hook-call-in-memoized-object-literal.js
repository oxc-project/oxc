import {useRef, useState} from 'react';

function useOptions(options) {
  return options;
}

// `useRef()` inside an object literal is called on every render, not only when
// the memoized object is created.
function Component({value}) {
  useOptions({countRef: useRef(0)});
  const [initial] = useState(value);
  return (
    <div>
      {initial}:{value}
    </div>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 1}],
  sequentialRenders: [{value: 1}, {value: 2}, {value: 2}],
};
