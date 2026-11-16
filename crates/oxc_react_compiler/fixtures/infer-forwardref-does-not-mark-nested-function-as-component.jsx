// @compilationMode:"infer"
import { forwardRef } from 'react';

const Grid = forwardRef(({ label }, ref) => {
  const renderGridComponent = () => <div ref={ref}>{label}</div>;
  return renderGridComponent();
});

const List = forwardRef(function ({ items }, ref) {
  const renderListComponent = () => <ul ref={ref}>{items}</ul>;
  return renderListComponent();
});

export { Grid, List };


const sproutInputs = [
  [{label:"first",items:[1,2]}],
  [{label:"second",items:[3]}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return <><Grid {...args[0]}/><List {...args[0]}/></>;
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
