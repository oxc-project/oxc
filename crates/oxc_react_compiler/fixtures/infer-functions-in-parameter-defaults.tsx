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
