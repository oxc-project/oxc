export function Row({ route }: { route: { useSearch: () => string } }) {
  "use no memo";
  const search = route.useSearch();
  return <div>{search}</div>;
}
