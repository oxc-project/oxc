import {createRef, forwardRef} from 'react';

const Component = forwardRef(
  (cb = () => ref.current++, ref = createRef()) => <div>{cb}</div>,
);
