// A global `useSharedValue` is an unknown hook, so writing its result is still
// reported.
export function SomeComponent() {
  const sharedVal = useSharedValue(0);
  return <Button onPress={() => (sharedVal.value = Math.random())} title="Randomize" />;
}
