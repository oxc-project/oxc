function Component(items: ReadonlyArray<number>) {
  let count = 0;
  return items.flatMap(() => () => count++);
}
