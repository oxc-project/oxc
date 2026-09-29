import {fbs} from 'fbt';

function Component(value) {
  switch (<fbs desc="Greeting">Hello</fbs>) {
    case 0:
      let fbs = value;
      return fbs;
    default:
      return null;
  }
}
