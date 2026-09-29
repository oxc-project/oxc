import {fbs} from 'fbt';

function Component() {
  function nested() {
    const fbs = () => null;
    return fbs;
  }
  return <fbs desc="Greeting">Hello</fbs>;
}
