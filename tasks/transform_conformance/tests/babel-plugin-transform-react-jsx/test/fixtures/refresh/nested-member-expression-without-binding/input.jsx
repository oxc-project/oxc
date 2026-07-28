export function App() {
  const foo = GlobalHook.property.useNestedThing();
  return <h1>{foo}</h1>;
}
