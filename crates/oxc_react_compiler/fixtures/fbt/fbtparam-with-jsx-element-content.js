import fbt from 'fbt';

function Component({name, data, icon}) {
  return (
    <Text type="body4">
      <fbt desc="Lorem ipsum">
        <fbt:param name="item author">
          <Text type="h4">{name}</Text>
        </fbt:param>
        <fbt:param name="icon">{icon}</fbt:param>
        <Text type="h4">
          <fbt:param name="item details">{data}</fbt:param>
        </Text>
      </fbt>
    </Text>
  );
}

import {Text} from 'shared-runtime';
const sproutInputs = [
  [{name:"Ada",data:"first",icon:"*"}],
  [{name:"Grace",data:"second",icon:"+"}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
