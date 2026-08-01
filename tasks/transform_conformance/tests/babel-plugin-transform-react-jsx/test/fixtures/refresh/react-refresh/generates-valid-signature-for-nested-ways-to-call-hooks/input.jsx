import FancyHook from 'fancy';

export default function App() {
  const foo = FancyHook.property.useNestedThing();
  return <h1>{foo}</h1>;
}
