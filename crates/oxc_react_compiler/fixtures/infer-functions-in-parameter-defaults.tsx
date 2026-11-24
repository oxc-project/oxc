// @compilationMode:"infer"
import { memo } from "react";

function createWrapper(Wrapper = memo(({ value }) => <div>{value}</div>)) {
  return Wrapper;
}

function createDestructuredWrapper({ Wrapper = memo(({ value }) => <span>{value}</span>) } = {}) {
  return Wrapper;
}

const factory = {
  create(Wrapper = memo(({ value }) => <p>{value}</p>)) {
    return Wrapper;
  },
};

export { createWrapper, createDestructuredWrapper, factory };


const sproutInputs = [
  [{value:"first"}],
  [{value:"second"}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return <>{React.createElement(createWrapper(),args[0])}{React.createElement(createDestructuredWrapper(),args[0])}{React.createElement(factory.create(),args[0])}</>;
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
