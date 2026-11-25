export const Example: FC<any> = (props) => {
  const {
    ['data-testid']: testId,
  } = props;
  return <div data-testid={testId}>Hello</div>;
};


const sproutInputs = [
  [{"data-testid":"first"}],
  [{"data-testid":"second"}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Example(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
