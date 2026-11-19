function Component({items}) {
  const colgroup = useMemo(
    () => (
      <colgroup>
        {items.map(item => (
          <col key={item.id} />
        ))}
      </colgroup>
    ),
    [items]
  );
  return (
    <table>
      {colgroup}
      <tbody />
    </table>
  );
}

import {useMemo} from 'react';
const sproutInputs = [
  [{items:[{id:1},{id:2}]}],
  [{items:[{id:3}]}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
