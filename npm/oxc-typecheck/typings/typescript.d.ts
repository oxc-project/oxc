// Source: https://github.com/typescript-eslint/typescript-eslint/blob/5a1e85da65cb83bc4e02965f8eb8f1f51347e004/packages/eslint-plugin/typings/typescript.d.ts

import 'typescript';

declare module 'typescript' {
  interface TypeChecker {
    // internal TS APIs

    getContextualTypeForArgumentAtIndex(node: Node, argIndex: number): Type;

    /**
     * @returns `true` if the given type is an array type:
     * - `Array<foo>`
     * - `ReadonlyArray<foo>`
     * - `foo[]`
     * - `readonly foo[]`
     */
    isArrayType(type: Type): type is TypeReference;
    /**
     * @returns `true` if the given type is a tuple type:
     * - `[foo]`
     * - `readonly [foo]`
     */
    isTupleType(type: Type): type is TupleTypeReference;
  }
}
