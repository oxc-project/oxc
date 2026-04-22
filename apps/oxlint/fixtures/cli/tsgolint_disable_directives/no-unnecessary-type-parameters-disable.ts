// This disable directive is USED even when the tsgolint diagnostic has labeled ranges.
// oxlint-disable-next-line typescript/no-unnecessary-type-parameters
export function parseYAML<T>(
  input: string,
): T {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion
  return input as any as T;
}
