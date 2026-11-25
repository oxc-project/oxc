// @compilationMode:"infer"
import { forwardRef } from 'react';

type Props = { text: string };

const TopStoriesItem = forwardRef<HTMLDivElement, Props>(
  ({ text }, viewTracker) => {
    return <div ref={viewTracker}>{text}</div>;
  },
);

export default TopStoriesItem;


const sproutInputs = [
  [{text:"first"}],
  [{text:"second"}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return React.createElement(TopStoriesItem, args[0]);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
