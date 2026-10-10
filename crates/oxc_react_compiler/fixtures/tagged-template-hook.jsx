import {useState} from 'react';

function useTagged(strings, value) {
  useState(0);
  return `${strings[0]}${value}${strings[1]}`;
}

function Component({value}) {
  const text = useTagged`value: ${value}`;
  return <div>{text}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 0}],
  sequentialRenders: [{value: 0}, {value: 0}],
};
