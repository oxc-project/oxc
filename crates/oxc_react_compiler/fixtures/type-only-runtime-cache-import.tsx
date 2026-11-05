import type { c as useMemoCache } from 'react/compiler-runtime';

function Component(props: { text: string }) {
  return <div>{props.text}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{ text: 'hello' }],
};
