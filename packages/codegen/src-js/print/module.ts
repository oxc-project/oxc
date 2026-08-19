// Modules.

import {
  CAT_IDENT,
  CAT_OTHER,
  CAT_START_OF_DEFAULT_EXPORT,
  markWithMapAtStartOffset,
  write,
  writeNoLast,
  writeWithMap,
} from "./write.ts";
import { printClass } from "./class.ts";
import { printExpression } from "./expression.ts";
import { printFunction } from "./function.ts";
import { printSpaceBeforeIdentifier } from "./space.ts";
import { printIndent } from "./indent.ts";
import { CTX_NONE } from "./operators.ts";
import { PREC_COMMA } from "./precedence.ts";
import { printVariableDeclaration } from "./statement.ts";
import { printString } from "./string.ts";
import {
  printTSEnumDeclaration,
  printTSImportEqualsDeclaration,
  printTSInterfaceDeclaration,
  printTSModuleDeclaration,
  printTSTypeAliasDeclaration,
} from "./typescript.ts";

import type { State } from "../state.ts";
import type { ExportNamedDeclarationNode } from "./types.ts";
import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/**
 * Print an `import` statement, in any of its forms.
 *
 * Default, namespace and named specifiers can be combined, and the named ones are the only kind in
 * braces, so the braces are opened and closed around a run rather than around the whole list.
 */
export function printImportDeclaration(node: ESTree.ImportDeclaration, state: State): void {
  printIndent(state);

  printSpaceBeforeIdentifier(state);
  writeWithMap(state, "import", CAT_IDENT, node);

  if (TS && node.importKind === "type") write(state, " type", CAT_IDENT);

  if (node.phase != null) {
    writeNoLast(state, " ");
    write(state, node.phase, CAT_IDENT);
  }

  const { specifiers } = node;
  const { length } = specifiers;
  if (length === 0) {
    // `import "source";`
    write(state, " ", CAT_OTHER);
    printString(state, node.source.value, node.source);
    printImportAttributes(node.attributes, state);
    write(state, ";\n", CAT_OTHER);
    return;
  }

  let inBlock = false;
  for (let i = 0; i < length; i++) {
    const specifier = specifiers[i];
    switch (specifier.type) {
      case "ImportDefaultSpecifier":
        if (inBlock) {
          write(state, " },", CAT_OTHER);
          inBlock = false;
        } else if (i === 0) {
          write(state, " ", CAT_OTHER);
        } else {
          write(state, ", ", CAT_OTHER);
        }

        printSpaceBeforeIdentifier(state);
        writeWithMap(state, specifier.local.name, CAT_IDENT, specifier);

        if (i === length - 1) write(state, " ", CAT_OTHER);

        break;
      case "ImportNamespaceSpecifier":
        if (inBlock) {
          write(state, " },", CAT_OTHER);
          inBlock = false;
        } else if (i === 0) {
          write(state, " ", CAT_OTHER);
        } else {
          write(state, ", ", CAT_OTHER);
        }

        write(state, "* as ", CAT_OTHER);
        writeWithMap(state, specifier.local.name, CAT_IDENT, specifier.local);
        write(state, " ", CAT_OTHER);
        break;
      default: {
        // ImportSpecifier
        if (inBlock) {
          write(state, ", ", CAT_OTHER);
        } else {
          if (i !== 0) {
            write(state, ",", CAT_OTHER);
          }
          inBlock = true;
          write(state, " { ", CAT_OTHER);
        }

        if (TS && specifier.importKind === "type") write(state, "type ", CAT_OTHER);

        const importedName = moduleExportName(specifier.imported, state);
        const { local } = specifier;
        if (importedName !== local.name) {
          write(state, " as ", CAT_OTHER);
          writeWithMap(state, local.name, CAT_IDENT, local);
        }
        break;
      }
    }
  }

  write(state, inBlock ? " } from " : "from ", CAT_OTHER);
  printString(state, node.source.value, node.source);
  printImportAttributes(node.attributes, state);
  write(state, ";\n", CAT_OTHER);
}

/**
 * Print a `with { … }` clause, and nothing at all when there are no attributes.
 *
 * An empty clause is written where the AST says the source had one, since `with {}`
 * and no clause are different programs.
 */
function printImportAttributes(
  attributes: ESTree.ImportAttribute[] | null | undefined,
  state: State,
): void {
  if (attributes == null) return;

  const { length } = attributes;
  if (length === 0) return;

  // ESTree omits the `WithClause` wrapper. The Rust reference normalizes its mapping anchor
  // to the first attribute, which is the first location both representations carry.
  writeNoLast(state, " ");
  markWithMapAtStartOffset(state, attributes[0], 0);
  write(state, "with { ", CAT_OTHER);

  for (let i = 0; i < length; i++) {
    if (i > 0) write(state, ", ", CAT_OTHER);

    const attribute = attributes[i];
    const { key } = attribute;
    if (key.type === "Identifier") {
      write(state, key.name, CAT_IDENT);
    } else {
      printString(state, key.value, key);
    }

    write(state, ": ", CAT_OTHER);

    printString(state, attribute.value.value, attribute.value);
  }

  write(state, " }", CAT_OTHER);
}

/**
 * Print the name on one side of an import or export specifier, and return it.
 *
 * The name comes back because the caller compares the two sides to decide whether an `as` clause
 * is needed at all, and it is cheaper to return it than to work it out twice.
 *
 * @returns The name printed
 */
function moduleExportName(node: ESTree.ModuleExportName, state: State): string {
  if (node.type === "Identifier") {
    printSpaceBeforeIdentifier(state);
    writeWithMap(state, node.name, CAT_IDENT, node);
    return node.name;
  }

  printString(state, node.value, node);
  return node.value;
}

/**
 * Print an `export` statement which names what it exports, whether that is a declaration,
 * a list of specifiers, or a list re-exported from another module.
 */
export function printExportNamedDeclaration(node: ExportNamedDeclarationNode, state: State): void {
  printIndent(state);

  writeWithMap(state, "export ", CAT_OTHER, node);

  const { declaration } = node;
  if (declaration != null) {
    switch (declaration.type) {
      case "VariableDeclaration":
        printVariableDeclaration(declaration, state, CTX_NONE);
        write(state, ";\n", CAT_OTHER);
        break;
      case "FunctionDeclaration":
      /* IF TS */
      case "TSDeclareFunction":
        /* END_IF */
        printFunction(declaration, state);
        write(state, "\n", CAT_OTHER);
        break;
      case "ClassDeclaration":
        printClass(declaration, state);
        write(state, "\n", CAT_OTHER);
        break;
      /* IF TS */
      case "TSModuleDeclaration":
        printTSModuleDeclaration(declaration, state);
        write(state, "\n", CAT_OTHER);
        break;
      case "TSInterfaceDeclaration":
        printTSInterfaceDeclaration(declaration, state);
        write(state, "\n", CAT_OTHER);
        break;
      case "TSEnumDeclaration":
        printTSEnumDeclaration(declaration, state);
        write(state, "\n", CAT_OTHER);
        break;
      case "TSTypeAliasDeclaration":
        printTSTypeAliasDeclaration(declaration, state);
        write(state, ";\n", CAT_OTHER);
        break;
      case "TSImportEqualsDeclaration":
        printTSImportEqualsDeclaration(declaration, state);
        write(state, ";\n", CAT_OTHER);
        break;
      /* END_IF */
      default:
        throw new Error(`Unknown export declaration type: ${declaration.type}`);
    }
    return;
  }

  if (TS && node.exportKind === "type") write(state, "type ", CAT_OTHER);

  write(state, "{", CAT_OTHER);

  const { specifiers } = node;
  const { length } = specifiers;
  if (length > 0) {
    write(state, " ", CAT_OTHER);
    for (let i = 0; i < length; i++) {
      if (i > 0) write(state, ", ", CAT_OTHER);
      const specifier = specifiers[i];
      if (TS && specifier.exportKind === "type") {
        write(state, "type ", CAT_OTHER);
      }
      const localName = moduleExportName(specifier.local, state);
      const exportedName =
        specifier.exported.type === "Identifier"
          ? specifier.exported.name
          : specifier.exported.value;
      if (localName !== exportedName) {
        write(state, " as ", CAT_OTHER);
        moduleExportName(specifier.exported, state);
      }
    }
    write(state, " ", CAT_OTHER);
  }

  write(state, "}", CAT_OTHER);

  if (node.source != null) {
    write(state, " from ", CAT_OTHER);
    printString(state, node.source.value, node.source);
    printImportAttributes(node.attributes, state);
  }

  write(state, ";\n", CAT_OTHER);
}

/**
 * Print `export * from "…"`, with the `as name` form where the AST has one.
 */
export function printExportAllDeclaration(node: ESTree.ExportAllDeclaration, state: State): void {
  printIndent(state);

  writeWithMap(
    state,
    TS && node.exportKind === "type" ? "export type *" : "export *",
    CAT_OTHER,
    node,
  );

  if (node.exported != null) {
    write(state, " as ", CAT_OTHER);
    moduleExportName(node.exported, state);
  }

  write(state, " from ", CAT_OTHER);
  printString(state, node.source.value, node.source);
  printImportAttributes(node.attributes, state);
  write(state, ";\n", CAT_OTHER);
}

/**
 * Print `export default ...`.
 *
 * The output position is recorded before the value is printed, so that a function or class expression
 * starting there knows to parenthesize itself.
 */
export function printExportDefaultDeclaration(
  node: ESTree.ExportDefaultDeclaration,
  state: State,
): void {
  printIndent(state);

  writeWithMap(state, "export default ", CAT_OTHER, node);

  const { declaration } = node;
  switch (declaration.type) {
    case "FunctionDeclaration":
    /* IF TS */
    case "TSDeclareFunction":
      /* END_IF */
      printFunction(declaration, state);
      write(state, "\n", CAT_OTHER);
      break;
    case "ClassDeclaration":
      printClass(declaration, state);
      write(state, "\n", CAT_OTHER);
      break;
    /* IF TS */
    case "TSInterfaceDeclaration":
      printTSInterfaceDeclaration(declaration, state);
      write(state, "\n", CAT_OTHER);
      break;
    /* END_IF */
    default:
      state.last = CAT_START_OF_DEFAULT_EXPORT;
      printExpression(declaration, state, PREC_COMMA, CTX_NONE);
      write(state, ";\n", CAT_OTHER);
      break;
  }
}
