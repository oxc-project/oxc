export const Example: FC<any> = (props) => {
  const {
    ['data-testid']: testId,
  } = props;
  return <div data-testid={testId}>Hello</div>;
};
