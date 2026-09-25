import {Stringify} from 'shared-runtime';

const NaN = 42;

function parameter(NaN) {
  return 1 ** (0 / 0);
}

function destructured({NaN}) {
  return 1 ** (1 / 0);
}

function local(props) {
  const NaN = props.value;
  return NaN + ((-1) ** (-1 / 0));
}

function outer() {
  return 1 ** (0 / 0);
}

function assignment(NaN) {
  let value = 1;
  value **= 0 / 0;
  return value;
}

function foo() {
  return (
    <Stringify
      value={[
        parameter(42),
        destructured({NaN: 42}),
        local({value: 42}),
        outer(),
        assignment(42),
      ]}
    />
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: foo,
  params: [],
  isComponent: false,
};
