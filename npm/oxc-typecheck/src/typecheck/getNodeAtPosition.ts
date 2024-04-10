// Source: https://github.com/microsoft/TypeScript/blob/25a708cf633c6c8a66c86ca9e664c31bd8d145d0/src/compiler/program.ts#L3448-L3462

import ts from 'typescript';
import { forEach, hasJSDocNodes } from './utils.js';

// TODO: consider obtaining array of child indexes directly from AST in Rust and just doing getChildAt(idx) instead
export function getNodeAtPosition(
  sourceFile: ts.SourceFile,
  line: number,
  character: number,
): ts.Node {
  // TODO: mapping line to position can be done in Rust
  const position = sourceFile.getPositionOfLineAndCharacter(line, character);

  const getContainingChild = (child: ts.Node): ts.Node | undefined => {
    if (
      child.pos <= position &&
      (position < child.end ||
        (position === child.end && child.kind === ts.SyntaxKind.EndOfFileToken))
    ) {
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

export function getParentOfKind(node: ts.Node, kind: ts.SyntaxKind): ts.Node {
  let current = node;
  while (current.kind !== kind && current.parent) {
    current = current.parent;
  }

  return current;
}
