export function Example() {
  return <Item node={<Child />} items={[]} callback={() => {}} options={{}} />;
}

export function Native() {
  return <div style={{ color: "red" }} />;
}

// oxlint-disable-next-line react-perf/jsx-no-new-array-as-prop
export function Suppressed() {
  return <Item items={[]} />;
}
