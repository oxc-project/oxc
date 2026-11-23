import {useIdentity} from 'shared-runtime';

// Reassigning a hoisted function after a frozen callback captured it is still
// reported; only the declaration's own initialization is allowed.
function Component() {
  const callback = useIdentity(() => format());
  format = () => 'reassigned';
  function format() {
    return 'declared';
  }
  return callback;
}
