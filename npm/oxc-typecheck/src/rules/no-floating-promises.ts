// Source: https://github.com/typescript-eslint/typescript-eslint/blob/a41ad155b5fee9177651439adb1c5131e7e6254f/packages/eslint-plugin/src/rules/no-floating-promises.ts

import * as tsutils from 'ts-api-utils';
import type * as ts from 'typescript';

export function isPromiseArray(
  checker: ts.TypeChecker,
  node: ts.Node,
): boolean {
  const type = checker.getTypeAtLocation(node);
  for (const ty of tsutils
    .unionTypeParts(type)
    .map((t) => checker.getApparentType(t))) {
    if (checker.isArrayType(ty)) {
      const arrayType = checker.getTypeArguments(ty)[0];
      if (isPromiseLike(checker, node, arrayType)) {
        return true;
      }
    }

    if (checker.isTupleType(ty)) {
      for (const tupleElementType of checker.getTypeArguments(ty)) {
        if (isPromiseLike(checker, node, tupleElementType)) {
          return true;
        }
      }
    }
  }
  return false;
}

// Modified from tsutils.isThenable() to only consider thenables which can be
// rejected/caught via a second parameter. Original source (MIT licensed):
//
//   https://github.com/ajafff/tsutils/blob/49d0d31050b44b81e918eae4fbaf1dfe7b7286af/util/type.ts#L95-L125
export function isPromiseLike(
  checker: ts.TypeChecker,
  node: ts.Node,
  type?: ts.Type,
): boolean {
  type ??= checker.getTypeAtLocation(node);
  for (const ty of tsutils.unionTypeParts(checker.getApparentType(type))) {
    const then = ty.getProperty('then');
    if (then === undefined) {
      continue;
    }

    const thenType = checker.getTypeOfSymbolAtLocation(then, node);
    if (
      hasMatchingSignature(
        thenType,
        (signature) =>
          signature.parameters.length >= 2 &&
          isFunctionParam(checker, signature.parameters[0], node) &&
          isFunctionParam(checker, signature.parameters[1], node),
      )
    ) {
      return true;
    }
  }
  return false;
}

function hasMatchingSignature(
  type: ts.Type,
  matcher: (signature: ts.Signature) => boolean,
): boolean {
  for (const t of tsutils.unionTypeParts(type)) {
    if (t.getCallSignatures().some(matcher)) {
      return true;
    }
  }

  return false;
}

function isFunctionParam(
  checker: ts.TypeChecker,
  param: ts.Symbol,
  node: ts.Node,
): boolean {
  const type: ts.Type | undefined = checker.getApparentType(
    checker.getTypeOfSymbolAtLocation(param, node),
  );
  for (const t of tsutils.unionTypeParts(type)) {
    if (t.getCallSignatures().length !== 0) {
      return true;
    }
  }
  return false;
}
