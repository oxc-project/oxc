// TypeScript (port of `gen.rs` TS sections).

import { typeAssertIs } from "../asserts.ts";
import {
  CAT_CLOSE_BRACKET,
  CAT_IDENT,
  CAT_LT,
  CAT_OP_UN_NOT,
  CAT_OTHER,
  CAT_QUESTION,
  debugAssertLastFresh,
  markWithMapAtStartOffset,
  write,
  writeNoLast,
  writeWithMap,
  writeWithMapEnd,
  writeWithMapNoLast,
} from "./write.ts";
import { printExpression } from "./expression.ts";
import { printParenParams, printParenParamsArrow } from "./function.ts";
import { printSpaceBeforeIdentifier } from "./space.ts";
import { printIndent } from "./indent.ts";
import { printLiteral } from "./literal.ts";
import { CTX_NONE, CTX_TYPESCRIPT } from "./operators.ts";
import {
  PREC_CALL,
  PREC_COMMA,
  PREC_COMPARE,
  PREC_EXPONENTIATION,
  PREC_LOWEST,
  PREC_PREFIX,
} from "./precedence.ts";
import { printDirectivesAndStatements } from "./statement.ts";
import { printString } from "./string.ts";

import type { State } from "../state.ts";
import type { LiteralExtras, UnknownNode } from "./types.ts";
import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/**
 * `as` and `satisfies` differ only in the keyword, so one function covers the pair and takes it from `node.type`.
 *
 * The operand prints at `PREC_EXPONENTIATION`, which leaves a unary operand unwrapped while still parenthesizing
 * anything looser, and the whole expression is wrapped once the surrounding precedence reaches `PREC_COMPARE`,
 * which is where `as` binds.
 */
export function printTSAsOrSatisfiesExpression(
  node: ESTree.TSAsExpression | ESTree.TSSatisfiesExpression,
  state: State,
  precedence: number,
  ctx: number,
): void {
  const wrap = precedence >= PREC_COMPARE;
  if (wrap) write(state, "(", CAT_OTHER);

  printExpression(node.expression, state, PREC_EXPONENTIATION, ctx);
  write(state, node.type === "TSAsExpression" ? " as " : " satisfies ", CAT_OTHER);
  printTSType(node.typeAnnotation, state);

  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * Dispatcher for type nodes. Also accepts tuple elements and qualified names, handing those to `printTSTupleElement`
 * and `printTSTypeName`, so a caller holding a field of either wider type needs no dispatch of its own.
 *
 * Keyword types are written as identifiers so a preceding keyword such as `extends` keeps its space.
 * A node type with no arm throws, rather than an unsupported AST printing silently wrong output.
 *
 * A prefix `TSJSDocNullableType` spaces itself off a preceding `?`, as `?` followed by `?T`
 * would otherwise merge into `??`.
 */
function printTSType(
  node: ESTree.TSTupleElement | ESTree.TSQualifiedName | UnknownNode,
  state: State,
): void {
  switch (node.type) {
    case "TSTypeReference":
      printTSTypeName(node.typeName, state);
      printTypeArguments(node.typeArguments, state);
      break;
    case "TSStringKeyword":
      printSpaceBeforeIdentifier(state);
      write(state, "string", CAT_IDENT);
      break;
    case "TSNumberKeyword":
      printSpaceBeforeIdentifier(state);
      write(state, "number", CAT_IDENT);
      break;
    case "TSBooleanKeyword":
      printSpaceBeforeIdentifier(state);
      write(state, "boolean", CAT_IDENT);
      break;
    case "TSAnyKeyword":
      printSpaceBeforeIdentifier(state);
      write(state, "any", CAT_IDENT);
      break;
    case "TSVoidKeyword":
      printSpaceBeforeIdentifier(state);
      write(state, "void", CAT_IDENT);
      break;
    case "TSUnknownKeyword":
      printSpaceBeforeIdentifier(state);
      write(state, "unknown", CAT_IDENT);
      break;
    case "TSNeverKeyword":
      printSpaceBeforeIdentifier(state);
      write(state, "never", CAT_IDENT);
      break;
    case "TSUndefinedKeyword":
      printSpaceBeforeIdentifier(state);
      write(state, "undefined", CAT_IDENT);
      break;
    case "TSNullKeyword":
      printSpaceBeforeIdentifier(state);
      write(state, "null", CAT_IDENT);
      break;
    case "TSObjectKeyword":
      printSpaceBeforeIdentifier(state);
      write(state, "object", CAT_IDENT);
      break;
    case "TSSymbolKeyword":
      printSpaceBeforeIdentifier(state);
      write(state, "symbol", CAT_IDENT);
      break;
    case "TSBigIntKeyword":
      printSpaceBeforeIdentifier(state);
      write(state, "bigint", CAT_IDENT);
      break;
    case "TSIntrinsicKeyword":
      printSpaceBeforeIdentifier(state);
      write(state, "intrinsic", CAT_IDENT);
      break;
    case "TSThisType":
      printSpaceBeforeIdentifier(state);
      write(state, "this", CAT_IDENT);
      break;
    case "TSLiteralType":
      printTSLiteral(node.literal, state);
      break;
    case "TSUnionType":
      printTSUnionType(node, state);
      break;
    case "TSIntersectionType":
      printTSIntersectionType(node, state);
      break;
    case "TSArrayType": {
      const wrap = parenthesizeTypeOfPostfixType(node.elementType);
      if (wrap) write(state, "(", CAT_OTHER);
      printTSType(node.elementType, state);
      if (wrap) writeNoLast(state, ")");
      write(state, "[]", CAT_CLOSE_BRACKET);
      break;
    }
    case "TSTypeLiteral":
      printTSTypeLiteral(node, state);
      break;
    case "TSFunctionType":
      printTSFunctionType(node, state);
      break;
    case "TSConstructorType":
      if (node.abstract) write(state, "abstract ", CAT_OTHER);
      write(state, "new ", CAT_OTHER);
      printTypeParameters(node.typeParameters, state);
      printParenParamsArrow(node.params, state);
      printTSType(tsTypeAnnotationOf(node.returnType), state);
      break;
    case "TSTupleType": {
      write(state, "[", CAT_OTHER);
      const { elementTypes } = node;
      const { length } = elementTypes;
      for (let i = 0; i < length; i++) {
        if (i > 0) write(state, ", ", CAT_OTHER);
        printTSTupleElement(elementTypes[i], state);
      }
      write(state, "]", CAT_CLOSE_BRACKET);
      break;
    }
    case "TSConditionalType":
      printTSConditionalType(node, state);
      break;
    case "TSInferType":
      write(state, "infer ", CAT_OTHER);
      printTSTypeParameter(node.typeParameter, state);
      break;
    case "TSIndexedAccessType": {
      const wrap = parenthesizeTypeOfPostfixType(node.objectType);
      if (wrap) write(state, "(", CAT_OTHER);
      printTSType(node.objectType, state);
      if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
      write(state, "[", CAT_OTHER);
      printTSType(node.indexType, state);
      write(state, "]", CAT_CLOSE_BRACKET);
      break;
    }
    case "TSMappedType":
      printTSMappedType(node, state);
      break;
    case "TSTypeOperator":
      printTSTypeOperator(node, state);
      break;
    case "TSTypePredicate":
      printTSTypePredicate(node, state);
      break;
    case "TSTypeQuery":
      write(state, "typeof ", CAT_OTHER);
      printTSTypeQueryExprName(node.exprName, state);
      printTypeArguments(node.typeArguments, state);
      break;
    case "TSImportType":
      printTSImportType(node, state);
      break;
    case "TSTemplateLiteralType": {
      writeNoLast(state, "`");
      const { quasis, types } = node;
      const { length } = quasis;
      for (let i = 0; i < length; i++) {
        if (i !== 0) {
          write(state, "${", CAT_OTHER);
          printTSType(types[i - 1], state);
          writeNoLast(state, "}");
        }
        writeNoLast(state, quasis[i].value.raw);
      }
      write(state, "`", CAT_OTHER);
      break;
    }
    case "TSParenthesizedType":
      write(state, "(", CAT_OTHER);
      printTSType(node.typeAnnotation, state);
      write(state, ")", CAT_CLOSE_BRACKET);
      break;
    case "TSNamedTupleMember":
      printTSTupleElement(node, state);
      break;
    case "TSQualifiedName":
      printTSTypeName(node, state);
      break;
    case "TSOptionalType":
    case "TSRestType":
      printTSTupleElement(node, state);
      break;
    case "TSJSDocUnknownType":
      printSpaceBeforeIdentifier(state);
      write(state, "unknown", CAT_IDENT);
      break;
    case "TSJSDocNullableType":
      if (node.postfix) {
        printTSType(tsTypeAnnotationOf(node.typeAnnotation), state);
        write(state, "?", CAT_QUESTION);
      } else {
        debugAssertLastFresh(state);
        if (state.last === CAT_QUESTION) write(state, " ", CAT_OTHER);
        write(state, "?", CAT_QUESTION);
        printTSType(tsTypeAnnotationOf(node.typeAnnotation), state);
      }
      break;
    case "TSJSDocNonNullableType":
      if (node.postfix) {
        printTSType(tsTypeAnnotationOf(node.typeAnnotation), state);
        write(state, "!", CAT_OP_UN_NOT);
      } else {
        write(state, "!", CAT_OP_UN_NOT);
        printTSType(tsTypeAnnotationOf(node.typeAnnotation), state);
      }
      break;
    default:
      throw new Error(`Unknown type node: ${node.type}`);
  }
}

/**
 * A dotted type name, printed by recursing down the `left` spine, so only the leftmost segment
 * takes the space-before-identifier check - every other segment follows a `.`.
 *
 * `this` is one of the names a qualified name can be rooted at, hence the `ThisExpression` arm.
 */
function printTSTypeName(
  node: ESTree.TSTypeName | ESTree.IdentifierName | ESTree.BindingIdentifier,
  state: State,
): void {
  if (node.type === "TSQualifiedName") {
    printTSTypeName(node.left, state);
    write(state, ".", CAT_OTHER);
    writeWithMap(state, node.right.name, CAT_IDENT, node.right);
  } else if (node.type === "ThisExpression") {
    printSpaceBeforeIdentifier(state);
    writeWithMap(state, "this", CAT_IDENT, node);
  } else {
    printSpaceBeforeIdentifier(state);
    writeWithMap(state, node.name, CAT_IDENT, node);
  }
}

/**
 * The `<...>` on a type reference, call, `new` expression or JSX element name.
 * Every one of those sites has the list as an optional field, so the null check lives here
 * rather than at each call site.
 */
export function printTypeArguments(
  typeArguments: ESTree.TSTypeParameterInstantiation | null | undefined,
  state: State,
): void {
  if (typeArguments == null) return;

  write(state, "<", CAT_LT);

  const { params } = typeArguments;
  const { length } = params;
  for (let i = 0; i < length; i++) {
    if (i > 0) write(state, ", ", CAT_OTHER);
    printTSType(params[i], state);
  }

  write(state, ">", CAT_OTHER);
}

/**
 * Literal types go through the expression printer under `CTX_TYPESCRIPT`, which makes a numeric literal
 * print its raw text instead of a reformatted value.
 *
 * A unary operand such as `-1` prints at `PREC_COMMA` precedence so it is not wrapped, as in Oxc.
 */
function printTSLiteral(literal: ESTree.TSLiteral, state: State): void {
  // A literal type holds a `Literal`, a `TemplateLiteral` or a `UnaryExpression` and nothing else,
  // and the first of those is what a literal type nearly always is, so it goes straight to its
  // printer rather than back through the expression dispatch.
  if (literal.type === "Literal") {
    printLiteral(literal, state, PREC_LOWEST, CTX_TYPESCRIPT);
  } else {
    printExpression(
      literal,
      state,
      literal.type === "UnaryExpression" ? PREC_COMMA : PREC_LOWEST,
      CTX_TYPESCRIPT,
    );
  }
}

/**
 * Members joined with `|`, each wrapped where `parenthesizeTypeOfUnionType` says so.
 * No leading `|` is printed, so a union node holding a single member prints as that member alone.
 */
function printTSUnionType(node: ESTree.TSUnionType, state: State): void {
  const { types } = node;
  const { length } = types;
  for (let i = 0; i < length; i++) {
    if (i > 0) write(state, " | ", CAT_OTHER);

    const wrap = parenthesizeTypeOfUnionType(types[i]);
    if (wrap) write(state, "(", CAT_OTHER);
    printTSType(types[i], state);
    if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
  }
}

/**
 * Whether a union member has to be parenthesized. Function, constructor, conditional and `infer` types
 * are wrapped because their bodies would otherwise run on past the `|`, and a nested union of more than
 * one member is wrapped as Oxc does.
 *
 * `oxc_codegen` decides here whether a type prints starting with a non-identifier character,
 * so a preceding keyword needs no separating space. In pretty mode a space is printed either way,
 * so nothing computes it.
 */
function parenthesizeTypeOfUnionType(ty: ESTree.TSType): boolean {
  switch (ty.type) {
    case "TSUnionType":
      return ty.types.length > 1;
    case "TSFunctionType":
    case "TSConstructorType":
    case "TSConditionalType":
    case "TSInferType":
      return true;
    default:
      return false;
  }
}

/**
 * The `&` counterpart of `printTSUnionType`, down to reusing the union's wrapping rule for members
 * which are not themselves intersections.
 */
function printTSIntersectionType(node: ESTree.TSIntersectionType, state: State): void {
  const { types } = node;
  const { length } = types;
  for (let i = 0; i < length; i++) {
    if (i > 0) write(state, " & ", CAT_OTHER);

    const wrap = parenthesizeTypeOfIntersectionType(types[i]);
    if (wrap) write(state, "(", CAT_OTHER);
    printTSType(types[i], state);
    if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
  }
}

/**
 * A nested intersection of more than one member is wrapped, and anything else falls through to
 * `parenthesizeTypeOfUnionType` - which is what parenthesizes a union inside an intersection, where
 * `&` binds tighter than `|`.
 */
function parenthesizeTypeOfIntersectionType(ty: ESTree.TSType): boolean {
  if (ty.type === "TSIntersectionType") return ty.types.length > 1;
  return parenthesizeTypeOfUnionType(ty);
}

/**
 * The wrapping rule for the operand of a postfix construct - `T[]`, `T[K]` and a tuple's `T?`. All of
 * those bind tighter than the listed types, so an unwrapped operand would re-parse with the postfix
 * applied to only its last member.
 */
function parenthesizeTypeOfPostfixType(ty: ESTree.TSType): boolean {
  switch (ty.type) {
    case "TSUnionType":
    case "TSIntersectionType":
    case "TSFunctionType":
    case "TSConstructorType":
    case "TSConditionalType":
    case "TSInferType":
    case "TSTypeOperator":
      return true;
    default:
      return false;
  }
}

/**
 * An empty type literal collapses to `{}`. Otherwise every member goes on its own indented line closed
 * with `;`, and the closing brace is indented back to the level of whatever contains it.
 */
function printTSTypeLiteral(node: ESTree.TSTypeLiteral, state: State): void {
  const { members } = node;
  const { length } = members;
  if (length === 0) {
    writeWithMapNoLast(state, "{", node);
    writeWithMapEnd(state, "}", CAT_OTHER, node);
    return;
  }

  writeWithMap(state, "{\n", CAT_OTHER, node);
  state.indentLevel++;

  for (let i = 0; i < length; i++) {
    printIndent(state);
    printTSSignature(members[i], state, CTX_TYPESCRIPT);
    write(state, ";\n", CAT_OTHER);
  }

  state.indentLevel--;
  printIndent(state);
  writeWithMapEnd(state, "}", CAT_OTHER, node);
}

/**
 * The members of a type literal or an interface body. `ctx` reaches the expression printer only through
 * computed keys and literal keys, which are the only places an expression can appear in a signature.
 *
 * Accessor signatures keep their `get`/`set` prefix, index signatures go to `printTSIndexSignature`,
 * and an unrecognized member type throws.
 */
function printTSSignature(node: ESTree.TSSignature | UnknownNode, state: State, ctx: number): void {
  switch (node.type) {
    case "TSPropertySignature":
      if (node.readonly) write(state, "readonly ", CAT_OTHER);
      if (node.computed) {
        write(state, "[", CAT_OTHER);
        typeAssertIs<ESTree.Expression>(node.key);
        printExpression(node.key, state, PREC_COMMA, ctx);
        write(state, "]", CAT_CLOSE_BRACKET);
      } else {
        printSignatureKey(node.key, state, ctx);
      }
      if (node.optional) write(state, "?", CAT_QUESTION);
      if (node.typeAnnotation != null) printTypeAnnotation(node.typeAnnotation, state);
      break;
    case "TSIndexSignature":
      printTSIndexSignature(node, state);
      break;
    case "TSMethodSignature":
      if (node.kind === "get") {
        write(state, "get ", CAT_OTHER);
      } else if (node.kind === "set") {
        write(state, "set ", CAT_OTHER);
      }

      if (node.computed) {
        write(state, "[", CAT_OTHER);
        typeAssertIs<ESTree.Expression>(node.key);
        printExpression(node.key, state, PREC_COMMA, ctx);
        write(state, "]", CAT_CLOSE_BRACKET);
      } else {
        printSignatureKey(node.key, state, ctx);
      }

      if (node.optional) write(state, "?", CAT_QUESTION);
      printTypeParameters(node.typeParameters, state);
      printParenParams(node.params, state);
      if (node.returnType != null) printTypeAnnotation(node.returnType, state);
      break;
    case "TSCallSignatureDeclaration":
      printTypeParameters(node.typeParameters, state);
      printParenParams(node.params, state);
      if (node.returnType != null) printTypeAnnotation(node.returnType, state);
      break;
    case "TSConstructSignatureDeclaration":
      write(state, "new ", CAT_OTHER);
      printTypeParameters(node.typeParameters, state);
      printParenParams(node.params, state);
      if (node.returnType != null) printTypeAnnotation(node.returnType, state);
      break;
    default:
      throw new Error(`Unknown signature type: ${node.type}`);
  }
}

/**
 * A signature's key.
 * A string key is re-printed by `printString` rather than passed to the expression printer,
 * so it comes out in the printer's own quotes instead of whatever the source used.
 */
function printSignatureKey(key: ESTree.PropertyKey, state: State, ctx: number): void {
  switch (key.type) {
    case "Identifier":
      printSpaceBeforeIdentifier(state);
      writeWithMap(state, key.name, CAT_IDENT, key);
      break;
    case "PrivateIdentifier":
      write(state, key.name, CAT_IDENT);
      break;
    case "Literal":
      typeAssertIs<LiteralExtras>(key);
      if (typeof key.value === "string") {
        printString(state, key.value, key);
      } else {
        printLiteral(key, state, PREC_COMMA, ctx);
      }
      break;
    default:
      printExpression(key, state, PREC_COMMA, ctx);
  }
}

/**
 * The `: T` half of an annotated binding, parameter, property or return type.
 * Takes the wrapper node the AST stores rather than the type inside it, so a caller
 * can pass the field straight through.
 */
export function printTypeAnnotation(annotation: ESTree.TSTypeAnnotation, state: State): void {
  write(state, ": ", CAT_OTHER);
  printTSType(tsTypeAnnotationOf(annotation), state);
}

/**
 * TS-ESTree wraps types in a `TSTypeAnnotation` node.
 * Fields vary in whether they hold that wrapper or a bare type, so every read which could be either
 * goes through here instead of the callers knowing which.
 */
function tsTypeAnnotationOf(annotation: ESTree.TSTypeAnnotation | ESTree.TSType): ESTree.TSType {
  return annotation.type === "TSTypeAnnotation" ? annotation.typeAnnotation : annotation;
}

/**
 * `[key: K]: V`, shared by type literals, interfaces and class bodies -
 * only the last of those can carry the `static` modifier.
 *
 * ESTree models the parameter as a list, but an index signature has exactly one -
 * Oxc's own AST stores a single parameter - so only the first entry is read.
 */
export function printTSIndexSignature(node: ESTree.TSIndexSignature, state: State): void {
  if (node.static) write(state, "static ", CAT_OTHER);
  if (node.readonly) write(state, "readonly ", CAT_OTHER);

  write(state, "[", CAT_OTHER);

  const parameter = node.parameters[0];
  write(state, parameter.name, CAT_IDENT);

  write(state, ": ", CAT_OTHER);

  printTSType(tsTypeAnnotationOf(parameter.typeAnnotation), state);

  write(state, "]: ", CAT_OTHER);

  printTSType(tsTypeAnnotationOf(node.typeAnnotation), state);
}

/**
 * The `<...>` on a declaration.
 * Two or more parameters print one per indented line while a single one stays inline,
 * which is Oxc's rule rather than a decision taken from the printed width.
 * As with `printTypeArguments`, a missing list is not an error.
 */
export function printTypeParameters(
  typeParameters: ESTree.TSTypeParameterDeclaration | null | undefined,
  state: State,
): void {
  if (typeParameters == null) return;

  const { params } = typeParameters;
  const { length } = params;
  const isMultiLine = length >= 2;

  write(state, "<", CAT_LT);

  if (isMultiLine) {
    state.indentLevel++;

    for (let i = 0; i < length; i++) {
      write(state, i !== 0 ? ",\n" : "\n", CAT_OTHER);
      printIndent(state);
      printTSTypeParameter(params[i], state);
    }

    write(state, "\n", CAT_OTHER);
    state.indentLevel--;
    printIndent(state);
  } else {
    for (let i = 0; i < length; i++) {
      if (i > 0) write(state, ", ", CAT_OTHER);
      printTSTypeParameter(params[i], state);
    }

    // `<T,>() => {}` - the comma stops it parsing as a JSX element
    if (state.isJsx) write(state, ",", CAT_OTHER);
  }

  write(state, ">", CAT_OTHER);
}

/**
 * One type parameter with its modifiers, constraint and default. The modifiers are independent flags, so
 * the fixed print order here - `const`, `in`, `out` - is what gives them an order at all.
 */
function printTSTypeParameter(node: ESTree.TSTypeParameter, state: State): void {
  // No `printSpaceBeforeIdentifier` needed, either here or before the name.
  //
  // All 3 callers leave `last` as `CAT_LT` or `CAT_OTHER`:
  // - `printTypeParameters` writes `<`, `, ` or a newline plus indent before each parameter.
  // - `TSInferType` writes `infer `.
  // - Each modifier below ends in a space.
  // So nothing an identifier could run into ever precedes one.
  //
  // With no reader of `last` left between the modifiers and the name, the modifiers need not record
  // themselves either - the name below is a real write, and always reached.

  if (node.const) writeNoLast(state, "const ");
  if (node.in) writeNoLast(state, "in ");
  if (node.out) writeNoLast(state, "out ");

  writeWithMap(state, node.name.name, CAT_IDENT, node.name);

  if (node.constraint != null) {
    write(state, " extends ", CAT_OTHER);
    printTSType(node.constraint, state);
  }

  if (node.default != null) {
    write(state, " = ", CAT_OTHER);
    printTSType(node.default, state);
  }
}

/**
 * `<T>(a: A) => R`.
 * The arrow comes from `printParenParamsArrow`, so the return type is printed as a bare type
 * rather than through `printTypeAnnotation`, which would put a `:` in front of it.
 */
function printTSFunctionType(node: ESTree.TSFunctionType, state: State): void {
  printTypeParameters(node.typeParameters, state);
  printParenParamsArrow(node.params, state);
  printTSType(tsTypeAnnotationOf(node.returnType), state);
}

/**
 * The forms only legal inside a tuple - `T?`, `...T` and a labelled `label: T`.
 * Anything else falls through to `printTSType`.
 *
 * The optional form borrows the postfix wrapping rule, since a trailing `?` binds as tightly as `[]`.
 */
function printTSTupleElement(node: ESTree.TSTupleElement, state: State): void {
  switch (node.type) {
    case "TSOptionalType": {
      const wrap = parenthesizeTypeOfPostfixType(node.typeAnnotation);
      if (wrap) write(state, "(", CAT_OTHER);
      printTSType(node.typeAnnotation, state);
      if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
      write(state, "?", CAT_QUESTION);
      break;
    }
    case "TSRestType":
      write(state, "...", CAT_OTHER);
      printTSType(node.typeAnnotation, state);
      break;
    case "TSNamedTupleMember":
      writeWithMap(state, node.label.name, CAT_IDENT, node.label);
      if (node.optional) write(state, "?", CAT_QUESTION);
      write(state, ": ", CAT_OTHER);
      printTSType(node.elementType, state);
      break;
    default:
      printTSType(node, state);
  }
}

/**
 * `C extends E ? T : F`. The check type is wrapped when it is a function, constructor or conditional
 * type, and the extends type only when it is itself conditional - otherwise its `?` would be read as
 * this conditional's.
 */
function printTSConditionalType(node: ESTree.TSConditionalType, state: State): void {
  const { checkType, extendsType } = node;
  const checkWrap =
    checkType.type === "TSFunctionType" ||
    checkType.type === "TSConstructorType" ||
    checkType.type === "TSConditionalType";

  if (checkWrap) write(state, "(", CAT_OTHER);
  printTSType(checkType, state);
  if (checkWrap) write(state, ")", CAT_CLOSE_BRACKET);

  write(state, " extends ", CAT_OTHER);

  const extendsWrapped = extendsType.type === "TSConditionalType";

  if (extendsWrapped) write(state, "(", CAT_OTHER);
  printTSType(extendsType, state);
  if (extendsWrapped) write(state, ")", CAT_CLOSE_BRACKET);

  write(state, " ? ", CAT_OTHER);
  printTSType(node.trueType, state);

  write(state, " : ", CAT_OTHER);
  printTSType(node.falseType, state);
}

/**
 * `{ [K in C]: T }`.
 * Both `readonly` and `?` are tri-state - present, added with `+`, or removed with `-` -
 * and the type annotation is optional.
 *
 * The braces are padded with spaces and the whole type stays on one line, however large it is.
 */
function printTSMappedType(node: ESTree.TSMappedType, state: State): void {
  writeNoLast(state, "{ ");

  const { readonly } = node;
  if (readonly === true) {
    writeNoLast(state, "readonly ");
  } else if (readonly === "+") {
    writeNoLast(state, "+readonly ");
  } else if (readonly === "-") {
    writeNoLast(state, "-readonly ");
  }

  writeNoLast(state, "[");

  writeWithMapNoLast(state, node.key.name, node.key);
  write(state, " in ", CAT_OTHER);
  printTSType(node.constraint, state);

  if (node.nameType != null) {
    write(state, " as ", CAT_OTHER);
    printTSType(node.nameType, state);
  }

  writeNoLast(state, "]");

  const { optional } = node;
  if (optional === true) {
    writeNoLast(state, "?");
  } else if (optional === "+") {
    writeNoLast(state, "+?");
  } else if (optional === "-") {
    writeNoLast(state, "-?");
  }

  if (node.typeAnnotation != null) {
    write(state, ": ", CAT_OTHER);
    printTSType(tsTypeAnnotationOf(node.typeAnnotation), state);
  }

  write(state, " }", CAT_OTHER);
}

/**
 * `keyof T`, `unique symbol` and `readonly T[]`. The operator is written as an identifier so a keyword
 * before it keeps its separating space, and the operand is wrapped where the operator would otherwise
 * bind to only part of it.
 */
function printTSTypeOperator(node: ESTree.TSTypeOperator, state: State): void {
  write(state, node.operator, CAT_IDENT);

  write(state, " ", CAT_OTHER);

  const ty = tsTypeAnnotationOf(node.typeAnnotation);
  const tyType = ty.type;
  const wrap =
    tyType === "TSUnionType" ||
    tyType === "TSIntersectionType" ||
    tyType === "TSFunctionType" ||
    tyType === "TSConstructorType" ||
    tyType === "TSConditionalType";

  if (wrap) write(state, "(", CAT_OTHER);
  printTSType(ty, state);
  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * `x is T`, with the optional `asserts` prefix.
 * The `is T` half is optional too, which is what makes a bare `asserts x` print.
 */
function printTSTypePredicate(node: ESTree.TSTypePredicate, state: State): void {
  if (node.asserts) write(state, "asserts ", CAT_OTHER);

  const { parameterName } = node;
  if (parameterName.type === "TSThisType") {
    write(state, "this", CAT_IDENT);
  } else {
    printSpaceBeforeIdentifier(state);
    writeWithMap(state, parameterName.name, CAT_IDENT, parameterName);
  }

  if (node.typeAnnotation != null) {
    write(state, " is ", CAT_OTHER);
    printTSType(tsTypeAnnotationOf(node.typeAnnotation), state);
  }
}

/**
 * The operand of `typeof`, which is a name in every case but `import("...")` -
 * an import type can stand where a name would.
 */
function printTSTypeQueryExprName(node: ESTree.TSTypeQueryExprName, state: State): void {
  if (node.type === "TSImportType") {
    printTSImportType(node, state);
  } else {
    printTSTypeName(node, state);
  }
}

/**
 * `import("mod", options).Q<T>`.
 *
 * The module specifier is re-printed as a string literal rather than copied,
 * so it comes out in the printer's own quotes.
 *
 * The options argument prints at `PREC_LOWEST` through the expression printer,
 * and both the qualifier and the type arguments are optional.
 */
function printTSImportType(node: ESTree.TSImportType, state: State): void {
  write(state, "import(", CAT_OTHER);

  printString(state, node.source.value, node.source);

  if (node.options != null) {
    write(state, ", ", CAT_OTHER);
    printExpression(node.options, state, PREC_LOWEST, CTX_TYPESCRIPT);
  }

  write(state, ")", CAT_CLOSE_BRACKET);

  if (node.qualifier != null) {
    write(state, ".", CAT_OTHER);
    printTSImportTypeQualifier(node.qualifier, state);
  }

  printTypeArguments(node.typeArguments, state);
}

/**
 * The dotted chain after `import(...)`.
 * Every segment follows a `.` written by the caller, so unlike `printTSTypeName`,
 * no space-before-identifier check is needed.
 */
function printTSImportTypeQualifier(node: ESTree.TSImportTypeQualifier, state: State): void {
  if (node.type === "TSQualifiedName") {
    printTSImportTypeQualifier(node.left, state);
    write(state, ".", CAT_OTHER);
    write(state, node.right.name, CAT_IDENT);
  } else {
    write(state, node.name, CAT_IDENT);
  }
}

/**
 * The `<T>x` cast form, which TypeScript rejects in `.tsx` files but which still has to print.
 *
 * A `<` written immediately before would merge into `<<`, so `last` is checked and a space inserted -
 * Oxc does the same by looking at the last byte written. The operand prints at `PREC_EXPONENTIATION`,
 * which leaves `<T>-x` unwrapped.
 */
export function printTSTypeAssertion(
  node: ESTree.TSTypeAssertion,
  state: State,
  precedence: number,
  ctx: number,
): void {
  const wrap = precedence >= PREC_PREFIX;
  if (wrap) write(state, "(", CAT_OTHER);

  debugAssertLastFresh(state);
  if (state.last === CAT_LT) write(state, " ", CAT_OTHER);

  write(state, "<", CAT_LT);

  // `< <T>() => T>x` requires a space to avoid `<<`
  if (node.typeAnnotation.type === "TSFunctionType") write(state, " ", CAT_OTHER);

  printTSType(node.typeAnnotation, state);
  write(state, ">", CAT_OTHER);
  printExpression(node.expression, state, PREC_EXPONENTIATION, ctx);

  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * `namespace X {}`, `module "x" {}` and `declare global {}`.
 * `global` is the one kind with no name of its own, so nothing is printed where the id would go.
 *
 * A missing body is the declaration form and ends in `;` instead of a block.
 */
export function printTSModuleDeclaration(
  node: ESTree.TSModuleDeclaration | ESTree.TSGlobalDeclaration,
  state: State,
): void {
  if (node.declare) write(state, "declare ", CAT_OTHER);

  const { kind } = node;
  write(state, kind, CAT_IDENT);

  if (kind !== "global") {
    write(state, " ", CAT_OTHER);
    const { id } = node;
    if (id.type === "Literal") {
      printString(state, id.value, id);
    } else {
      printTSTypeName(id, state);
    }
  }

  const { body } = node;
  if (body == null) {
    write(state, ";", CAT_OTHER);
    return;
  }

  write(state, " ", CAT_OTHER);

  printModuleBlock(body, state);
}

/**
 * The body of a namespace or module. It goes through `printDirectivesAndStatements` rather than a
 * plain statement loop, so a directive prologue inside a module block is still recognized.
 */
function printModuleBlock(body: ESTree.TSModuleBlock, state: State): void {
  const statements = body.body;
  if (statements.length === 0) {
    writeWithMapNoLast(state, "{", body);
    writeWithMapEnd(state, "}", CAT_OTHER, body);
    return;
  }

  writeWithMap(state, "{\n", CAT_OTHER, body);

  state.indentLevel++;
  printDirectivesAndStatements(statements, state);
  state.indentLevel--;

  printIndent(state);
  writeWithMapEnd(state, "}", CAT_OTHER, body);
}

/**
 * `interface X<T> extends A, B { ... }`.
 * A heritage clause is an expression plus optional type arguments rather than a type,
 * so its expression prints at `PREC_CALL` precedence through the expression printer.
 *
 * The body uses the same one-member-per-line layout as `printTSTypeLiteral`,
 * including the `{}` form when there are no members.
 */
export function printTSInterfaceDeclaration(
  node: ESTree.TSInterfaceDeclaration,
  state: State,
): void {
  printSpaceBeforeIdentifier(state);

  if (node.declare) write(state, "declare ", CAT_OTHER);

  write(state, "interface ", CAT_OTHER);

  writeWithMap(state, node.id.name, CAT_IDENT, node.id);

  printTypeParameters(node.typeParameters, state);

  const extendsClauses = node.extends;
  if (extendsClauses != null && extendsClauses.length > 0) {
    write(state, " extends ", CAT_OTHER);

    const { length } = extendsClauses;
    for (let i = 0; i < length; i++) {
      if (i > 0) write(state, ", ", CAT_OTHER);
      const clause = extendsClauses[i];
      printExpression(clause.expression, state, PREC_CALL, CTX_NONE);
      printTypeArguments(clause.typeArguments, state);
    }
  }

  write(state, " ", CAT_OTHER);

  const members = node.body.body;
  const { length } = members;
  if (length === 0) {
    writeWithMapNoLast(state, "{", node.body);
    writeWithMapEnd(state, "}", CAT_OTHER, node.body);
    return;
  }

  writeWithMap(state, "{\n", CAT_OTHER, node.body);
  state.indentLevel++;

  for (let i = 0; i < length; i++) {
    printIndent(state);
    printTSSignature(members[i], state, CTX_NONE);
    write(state, ";\n", CAT_OTHER);
  }

  state.indentLevel--;
  printIndent(state);
  writeWithMapEnd(state, "}", CAT_OTHER, node.body);
}

/**
 * `type X<T> = ...`, including the `declare` form.
 * An alias body is the one position where a type can need parentheses purely because of what it starts with -
 * see `isLeftmostIntrinsicReference`.
 */
export function printTSTypeAliasDeclaration(
  node: ESTree.TSTypeAliasDeclaration,
  state: State,
): void {
  if (node.declare) write(state, "declare ", CAT_OTHER);

  write(state, "type ", CAT_OTHER);

  writeWithMap(state, node.id.name, CAT_IDENT, node.id);

  printTypeParameters(node.typeParameters, state);

  write(state, " = ", CAT_OTHER);

  // A leftmost bare `intrinsic` reference must keep parentheses, otherwise it
  // re-parses as the `intrinsic` keyword.
  const needsParens = isLeftmostIntrinsicReference(node.typeAnnotation);
  if (needsParens) write(state, "(", CAT_OTHER);
  printTSType(node.typeAnnotation, state);
  if (needsParens) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * Whether the leftmost type printed in an alias body is a reference to a type named `intrinsic`,
 * which TypeScript reads as the `intrinsic` keyword unless it is parenthesized.
 * A reference carrying type arguments cannot be read as the keyword, so it does not count.
 *
 * The leftmost edge is walked with a loop rather than recursion - through array and indexed-access
 * object types, the first member of a union or intersection, and a conditional's check type.
 */
function isLeftmostIntrinsicReference(ty: ESTree.TSType): boolean {
  for (;;) {
    switch (ty.type) {
      case "TSTypeReference":
        return (
          ty.typeArguments == null &&
          ty.typeName.type === "Identifier" &&
          ty.typeName.name === "intrinsic"
        );
      case "TSArrayType":
        ty = ty.elementType;
        break;
      case "TSIndexedAccessType":
        ty = ty.objectType;
        break;
      case "TSUnionType":
      case "TSIntersectionType":
        if (ty.types.length === 0) return false;
        ty = ty.types[0];
        break;
      case "TSConditionalType":
        ty = ty.checkType;
        break;
      default:
        return false;
    }
  }
}

/**
 * `enum X { ... }` with its `declare` and `const` modifiers.
 * Members print one per indented line with a comma after all but the last, and an enum with no members
 * collapses to `{}`.
 *
 */
export function printTSEnumDeclaration(node: ESTree.TSEnumDeclaration, state: State): void {
  printSpaceBeforeIdentifier(state);

  if (node.declare) write(state, "declare ", CAT_OTHER);
  if (node.const) write(state, "const ", CAT_OTHER);

  write(state, "enum ", CAT_OTHER);

  writeWithMap(state, node.id.name, CAT_IDENT, node.id);

  write(state, " ", CAT_OTHER);

  const { body } = node;
  const { members } = body;
  const { length } = members;
  if (length === 0) {
    writeWithMapNoLast(state, "{", body);
    writeWithMapEnd(state, "}", CAT_OTHER, body);
    return;
  }

  writeWithMap(state, "{\n", CAT_OTHER, body);
  state.indentLevel++;

  const lastIndex = length - 1;
  for (let i = 0; i < length; i++) {
    printIndent(state);
    printTSEnumMember(members[i], state);
    write(state, i !== lastIndex ? ",\n" : "\n", CAT_OTHER);
  }

  state.indentLevel--;
  printIndent(state);
  writeWithMapEnd(state, "}", CAT_OTHER, body);
}

/**
 * A member name is written bare when it is an identifier and quoted when it is a string, so `A` and `"a"`
 * stay distinct. A template name prints only its first quasi, as such a name has to be a constant string
 * and so has no substitutions.
 */
function printTSEnumMember(node: ESTree.TSEnumMember, state: State): void {
  const { id } = node;
  if (id.type === "Identifier") {
    printSpaceBeforeIdentifier(state);
    writeWithMap(state, id.name, CAT_IDENT, id);
  } else if (id.type === "Literal") {
    printString(state, id.value, id);
  } else {
    // Computed string/template member name
    if (id.type === "TemplateLiteral") {
      markWithMapAtStartOffset(state, id.quasis[0], 1);
      writeNoLast(state, "[`");
      writeNoLast(state, id.quasis[0].value.raw);
      write(state, "`", CAT_OTHER);
    } else {
      write(state, "[", CAT_OTHER);
      printExpression(id, state, PREC_COMMA, CTX_NONE);
    }
    write(state, "]", CAT_CLOSE_BRACKET);
  }

  if (node.initializer != null) {
    write(state, " = ", CAT_OTHER);
    printExpression(node.initializer, state, PREC_LOWEST, CTX_NONE);
  }
}

/**
 * `import X = require("mod")` and `import X = A.B`, which share everything up to the `=`.
 *
 * The `type` modifier goes after `import`, as `import type X = require("mod")` is the spelling TypeScript accepts.
 */
export function printTSImportEqualsDeclaration(
  node: ESTree.TSImportEqualsDeclaration,
  state: State,
): void {
  write(state, "import ", CAT_OTHER);

  if (node.importKind === "type") write(state, "type ", CAT_OTHER);

  writeWithMap(state, node.id.name, CAT_IDENT, node.id);

  write(state, " = ", CAT_OTHER);

  const ref = node.moduleReference;
  if (ref.type === "TSExternalModuleReference") {
    write(state, "require(", CAT_OTHER);
    printString(state, ref.expression.value, ref.expression);
    write(state, ")", CAT_CLOSE_BRACKET);
  } else {
    printTSTypeName(ref, state);
  }
}
