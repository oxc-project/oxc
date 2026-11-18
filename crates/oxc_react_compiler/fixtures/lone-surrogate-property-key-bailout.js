// @compilationMode:"infer" @panicThreshold:"none"

export function Component() {
  return <div>{({"\uD800": 1})["\uD800"]}</div>;
}

export function TrailSurrogate() {
  return <div>{({"\uDC00": 2})["\uDC00"]}</div>;
}

export const Arrow = () => <div>{({"\uD800": 3})["\uD800"]}</div>;

export function ComputedKey() {
  return <div>{({["\uD800"]: 4})["\uD800"]}</div>;
}

export function DestructuredKey({"\uD800": value}) {
  return <div>{value}</div>;
}

export function AssignedKey(props) {
  let value;
  ({"\uDC00": value} = props);
  return <div>{value}</div>;
}

export function MethodKey() {
  const object = {"\uD800"() { return 5; }};
  return <div>{object["\uD800"]()}</div>;
}

export function NestedFunction() {
  const read = () => {
    return ({"\uDC00": 6})["\uDC00"];
  };
  return <div>{read()}</div>;
}

export function SupportedKeys() {
  return <div>{({"\uD800\uDC00": 7, "\uFFFDd800": 8})["\uD800\uDC00"]}</div>;
}


const sproutInputs = [
  [{"\uD800":10,"\uDC00":11}],
  [{"\uD800":12,"\uDC00":13}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return [Component(), TrailSurrogate(), Arrow(), ComputedKey(), DestructuredKey(args[0]), AssignedKey(args[0]), MethodKey(), NestedFunction(), SupportedKeys()];
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
