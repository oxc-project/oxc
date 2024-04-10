import type ts from 'typescript';

export function forEach<T, U>(
  array: readonly T[] | undefined,
  callback: (element: T, index: number) => U | undefined,
): U | undefined {
  if (array) {
    for (let i = 0; i < array.length; i++) {
      const result = callback(array[i], i);
      if (result) {
        return result;
      }
    }
  }
  return undefined;
}

export function hasJSDocNodes(
  node: ts.Node,
): node is ts.HasJSDoc & { jsDoc: ts.Node[] } {
  if (!('jsDoc' in node)) {
    return false;
  }

  const { jsDoc } = node as { jsDoc?: ts.Node[] };
  return !!jsDoc && jsDoc.length > 0;
}
