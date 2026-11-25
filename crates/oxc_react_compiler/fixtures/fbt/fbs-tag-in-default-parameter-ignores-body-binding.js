import {fbs} from 'fbt';

function Component(text = <fbs desc="Greeting">Hello</fbs>) {
  function fbs() {}
  return <div>{text}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [],
};
