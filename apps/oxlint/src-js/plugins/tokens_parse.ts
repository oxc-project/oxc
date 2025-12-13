/*
 * Use TypeScript parser to get tokens from source text.
 */

import { createRequire } from "node:module";
import { filePath } from "./context.ts";
import { getNodeLoc } from "./location.ts";
import { sourceText } from "./source_code.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

import type * as ts from "typescript";
import type { Token } from "./tokens.ts";

// `typescript` package is lazy-loaded only when needed, as it's a lot of code.
//
// `tsSyntaxKind` is `null` originally.
// Using hack of `null as any` to avoid having to assert that `tsSyntaxKind` is not `null` all over the place.
// `initTokens` initializes `tsSyntaxKind`, before any of the functions that access `tsSyntaxKind` are called.
let tsModule: typeof import("typescript") | null = null;
let tsSyntaxKind: typeof import("typescript").SyntaxKind = null as any;

// Prototype for `Token` objects, which calculates `loc` property lazily.
// This is the same as `NodeProto` used for AST nodes (in `generated/deserialize.js`).
// TODO: De-duplicate this code between here and `generated/deserialize.js`.
const TokenProto = Object.create(null, {
  loc: {
    get() {
      return getNodeLoc(this);
    },
    enumerable: true,
  },
});

/**
 * Initialize TS-ESLint tokens for current file.
 *
 * Caller must ensure `filePath` and `sourceText` are initialized before calling this function.
 */
export function parseTokens(): Token[] {
  debugAssertIsNonNull(filePath);
  debugAssertIsNonNull(sourceText);

  // Lazy-load TypeScript.
  // `./typescript.cjs` is path to the bundle in `dist` directory, as well as relative path in `src-js`,
  // so is valid both in bundled `dist` output, and in unit tests.
  if (tsModule === null) {
    const require = createRequire(import.meta.url);
    tsModule = require("./typescript.cjs");
    debugAssertIsNonNull(tsModule);
    tsSyntaxKind = tsModule.SyntaxKind;
  }

  // Parse source text into TypeScript AST
  const tsAst = tsModule.createSourceFile(
    filePath,
    sourceText,
    // These options are the same as in TS-ESLint's `parse` function
    {
      jsDocParsingMode: tsModule.JSDocParsingMode.ParseNone,
      languageVersion: tsModule.ScriptTarget.Latest,
      setExternalModuleIndicator: undefined,
    },
    true, // `setParentNodes`
    // TODO: Use `TS` or `TSX` depending on source type
    tsModule.ScriptKind.TSX,
  );

  // Check that TypeScript hasn't altered source text.
  // If it had, token ranges would be incorrect.
  debugAssert(tsAst.text === sourceText);

  // Extract tokens from TypeScript AST
  return convertTokens(tsAst);
}

/**
 * Convert all tokens for the given AST.
 *
 * Adapted from:
 * https://github.com/typescript-eslint/typescript-eslint/blob/5bd78cab52569a5e9e14a8e4588a672ca933a0be/packages/typescript-estree/src/node-utils.ts#L617-L637
 *
 * This function and all functions below in this file are copied from TS-ESLint's implementation,
 * and refactored to reduce function calls and duplicated property lookups.
 *
 * Only substantive differences are:
 * 1. `Token`s we produce have `start` and `end` properties, whereas TS-ESLint's `Token`s do not.
 * 2. We lazily calculate `loc`, whereas TS-ESLint does it eagerly.
 * 3. Added workaround for TypeScript not being able to parse JSX closing fragment containing a line break.
 *
 * @param tsAst - TypeScript AST
 * @returns Array of `Token`s
 */
function convertTokens(tsAst: ts.SourceFile): Token[] {
  const tokens: Token[] = [];
  walk(tsAst);
  return tokens;

  function walk(node: ts.Node): void {
    const { kind } = node;

    // TypeScript generates tokens for types in JSDoc blocks.
    // Comment tokens and their children should not be walked or added to the resulting tokens list.
    if (
      kind === tsSyntaxKind.SingleLineCommentTrivia ||
      kind === tsSyntaxKind.MultiLineCommentTrivia ||
      kind === tsSyntaxKind.JSDoc
    ) {
      return;
    }

    if (
      kind >= tsSyntaxKind.FirstToken &&
      kind <= tsSyntaxKind.LastToken &&
      kind !== tsSyntaxKind.EndOfFileToken
    ) {
      // Token
      convertToken(node as ts.Token<ts.TokenSyntaxKind>, tsAst, tokens);
    } else {
      // Node
      node.getChildren(tsAst).forEach(walk);
    }
  }
}

/**
 * Convert TS token to TS-ESLint-style token.
 * @param token - TS token
 * @param tsAst - TypeScript AST
 * @param tokens - Array of tokens to push converted token to
 */
function convertToken(token: ts.Token<ts.TokenSyntaxKind>, tsAst: ts.SourceFile, tokens: Token[]) {
  const { kind } = token;

  const start = kind === tsSyntaxKind.JsxText ? token.getFullStart() : token.getStart(tsAst);
  const end = token.getEnd();

  // TypeScript cannot parse JSX closing fragment containing a line break. e.g. `<><\n/>`
  // It produces an `Identifier` token with 0-length range. Skip these invalid tokens.
  if (start === end) return;

  let value = sourceText!.slice(start, end);

  if (kind === tsSyntaxKind.RegularExpressionLiteral) {
    tokens.push({
      // `TokenProto` provides getter for `loc`
      // @ts-expect-error - TS doesn't understand `__proto__`
      __proto__: TokenProto,
      type: "RegularExpression",
      value,
      regex: {
        flags: value.slice(value.lastIndexOf("/") + 1),
        pattern: value.slice(1, value.lastIndexOf("/")),
      },
      start,
      end,
      range: [start, end],
    });
  } else {
    const tokenType = getTokenType(token);

    // TODO: `kind === tsSyntaxKind.PrivateIdentifier` would be a faster check, but TS doesn't like it
    if (tokenType === "PrivateIdentifier") value = value.slice(1);

    tokens.push({
      // `TokenProto` provides getter for `loc`
      // @ts-expect-error - TS doesn't understand `__proto__`
      __proto__: TokenProto,
      type: tokenType,
      value,
      start,
      end,
      range: [start, end],
    });
  }
}

/**
 * Returns the type of a given `ts.Token`.
 */
function getTokenType(token: ts.Identifier | ts.Token<ts.SyntaxKind>): Token["type"] {
  const { kind } = token;

  if (kind === tsSyntaxKind.NullKeyword) return "Null";

  if (kind >= tsSyntaxKind.FirstKeyword && kind <= tsSyntaxKind.LastFutureReservedWord) {
    return kind === tsSyntaxKind.FalseKeyword || kind === tsSyntaxKind.TrueKeyword
      ? "Boolean"
      : "Keyword";
  }

  if (kind >= tsSyntaxKind.FirstPunctuation && kind <= tsSyntaxKind.LastPunctuation) {
    return "Punctuator";
  }

  if (kind >= tsSyntaxKind.NoSubstitutionTemplateLiteral && kind <= tsSyntaxKind.TemplateTail) {
    return "Template";
  }

  switch (kind) {
    case tsSyntaxKind.NumericLiteral:
    case tsSyntaxKind.BigIntLiteral:
      return "Numeric";

    case tsSyntaxKind.PrivateIdentifier:
      return "PrivateIdentifier";

    case tsSyntaxKind.JsxText:
      return "JSXText";

    case tsSyntaxKind.StringLiteral: {
      // A TypeScript `StringLiteral` token with a `JsxAttribute` or `JsxElement` parent,
      // must actually be an ESTree `JSXText` token
      const parentKind = token.parent.kind;
      return parentKind === tsSyntaxKind.JsxAttribute || parentKind === tsSyntaxKind.JsxElement
        ? "JSXText"
        : "String";
    }

    case tsSyntaxKind.RegularExpressionLiteral:
      return "RegularExpression";

    case tsSyntaxKind.Identifier: {
      // Some JSX tokens have to be determined based on their parent
      const { parent } = token,
        parentKind = parent.kind;
      if (
        isJSXTokenKind(parentKind) ||
        (parentKind === tsSyntaxKind.PropertyAccessExpression && hasJSXAncestor(parent.parent))
      ) {
        return "JSXIdentifier";
      }
    }

    /*
    case tsSyntaxKind.ConstructorKeyword:
    case tsSyntaxKind.GetKeyword:
    case tsSyntaxKind.SetKeyword:

    // Intentional fallthrough
    default:
    */
  }

  return "Identifier";
}

/**
 * Check if `node` is a JSX token, or has a JSX token within its ancestry.
 * @param node - TS AST node
 * @returns `true` if `node` has a JSX token within its ancestry
 */
function hasJSXAncestor(node: ts.Node | undefined): boolean {
  while (node !== undefined) {
    if (isJSXTokenKind(node.kind)) return true;
    node = node.parent;
  }

  return false;
}

/**
 * Check if `kind` is a JSX token kind.
 * @param kind - TS AST token kind
 * @returns `true` if `kind` is a JSX token kind
 */
function isJSXTokenKind(kind: ts.SyntaxKind): boolean {
  return kind >= tsSyntaxKind.JsxElement && kind <= tsSyntaxKind.JsxAttribute;
}
