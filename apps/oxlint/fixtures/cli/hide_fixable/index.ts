export function example(value: string) {
  if (typeof value == "undefined") return;
  if (value == "value") return;
  debugger;
  const unused = 1;
  const redundant = value as string;
  Promise.resolve(redundant);
}
