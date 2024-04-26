// Source: https://github.com/microsoft/TypeScript/blob/25a708cf633c6c8a66c86ca9e664c31bd8d145d0/src/compiler/program.ts#L3448-L3462

import ts from 'typescript';
import { forEach, hasJSDocNodes } from './utils.js';

const cache = new Map<string, { pos: number; end: number; node: ts.Node }>();

// TODO: consider obtaining array of child indexes directly from AST in Rust and just doing getChildAt(idx) instead
function searchNodeAtPosition(
  sourceFile: ts.SourceFile,
  pos: number,
  end: number,
): ts.Node {
  const getContainingChild = (child: ts.Node): ts.Node | undefined => {
    if (child.pos <= pos && end <= child.end) {
      return child;
    }

    return;
  };

  let current: ts.Node = sourceFile;
  while (true) {
    const child =
      (sourceFile.fileName.endsWith('.js') &&
        hasJSDocNodes(current) &&
        forEach(current.jsDoc, getContainingChild)) ||
      ts.forEachChild(current, getContainingChild);
    if (!child) {
      return current;
    }
    current = child;
  }
}

export function getNodeAtPosition(
  sourceFile: ts.SourceFile,
  { pos, end }: ts.ReadonlyTextRange,
): ts.Node {
  const cachedNode = cache.get(sourceFile.fileName);
  if (cachedNode && cachedNode.pos === pos && cachedNode.end === end) {
    return cachedNode.node;
  }

  const node = searchNodeAtPosition(sourceFile, pos, end);
  if (cachedNode) {
    cachedNode.pos = pos;
    cachedNode.end = end;
    cachedNode.node = node;
  } else {
    cache.set(sourceFile.fileName, { pos, end, node });
  }

  return node;
}

export function deleteNodeCache(fileName: string) {
  cache.delete(fileName);
}
