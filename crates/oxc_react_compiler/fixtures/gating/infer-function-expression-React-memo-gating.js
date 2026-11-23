// @gating @compilationMode:"infer"
import React from 'react';
const SproutComponent = React.forwardRef(function notNamedLikeAComponent(props) {
  return <div />;
});

export default SproutComponent;

const sproutInputs = [
  [],
  [],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return React.createElement(SproutComponent, args[0]);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
