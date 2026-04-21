// Examples of incorrect code for no-unnecessary-type-parameters rule

function parseYAML<T>(input: string): T {
  return input as any as T;
}
