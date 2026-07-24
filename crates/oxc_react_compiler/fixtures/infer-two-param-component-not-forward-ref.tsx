// @compilationMode:"infer"
type Props = {label: string};

const Card = ({label}: Props, viewTracker: unknown) => (
  <div data-view-tracker={String(viewTracker)}>{label}</div>
);

export default Card;
