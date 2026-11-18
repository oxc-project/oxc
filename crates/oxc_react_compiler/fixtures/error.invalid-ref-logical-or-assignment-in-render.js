// @validateRefAccessDuringRender
import {useRef} from 'react';

// `||=` is not the nullish lazy-init pattern: it also assigns when current
// is a legitimate falsy value such as 0 or false.
function Component(props) {
  const ref = useRef(null);
  ref.current ||= props.value;
  return ref;
}
