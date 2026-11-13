function HomeDiscoStoreItemTileRating(props) {
  const item = useFragment();
  let count = 0;
  const aggregates = item?.aggregates || [];
  aggregates.forEach(aggregate => {
    count += aggregate.count || 0;
  });

  return <Text>{count}</Text>;
}

import {useFragment, Text} from 'shared-runtime';
const sproutInputs = [
  [{}],
  [{}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return HomeDiscoStoreItemTileRating(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
