// @compilationMode:"infer"
import { forwardRef } from 'react';

type Props = { text: string };

const TopStoriesItem = forwardRef<HTMLDivElement, Props>(
  ({ text }, viewTracker) => {
    return <div ref={viewTracker}>{text}</div>;
  },
);

export default TopStoriesItem;
