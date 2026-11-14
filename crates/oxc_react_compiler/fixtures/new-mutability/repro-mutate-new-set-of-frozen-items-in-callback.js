// @enableNewMutationAliasingModel:true

export const App = () => {
  const [selected, setSelected] = useState(new Set<string>());
  const onSelectedChange = (value: string) => {
    const newSelected = new Set(selected);
    if (newSelected.has(value)) {
      // This should not count as a mutation of `selected`
      newSelected.delete(value);
    } else {
      // This should not count as a mutation of `selected`
      newSelected.add(value);
    }
    setSelected(newSelected);
  };

  return <Stringify selected={selected} onSelectedChange={onSelectedChange} />;
};

import {useState} from 'react';
import {Stringify} from 'shared-runtime';
const sproutInputs = [
  [],
  [],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return App(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
