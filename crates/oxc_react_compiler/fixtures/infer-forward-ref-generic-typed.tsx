// @compilationMode:"infer"
import {forwardRef} from 'react';

type Props = {label: string};

const Card = forwardRef<HTMLDivElement, Props>(
  ({label}: Props, viewTracker) => (
    <div ref={viewTracker}>{label}</div>
  ),
);

export default Card;
