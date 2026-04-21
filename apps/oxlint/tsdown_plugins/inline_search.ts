import fs from "node:fs";
import { join as pathJoin } from "node:path";
import { Visitor } from "oxc-parser";
import { parse } from "./utils.ts";

import type { Plugin } from "rolldown";
import type * as ESTree from "@oxc-project/types";

// Name of binary search function to inline
const INLINE_FUNC_NAME = "firstTokenAtOrAfter";

// Path of file the binary search function is defined in
const INLINE_FUNC_PATH = pathJoin(import.meta.dirname, "../src-js/plugins/tokens_methods.ts");

// Files to inline the binary search function into
const FILES = ["/src-js/plugins/tokens_methods.ts", "/src-js/plugins/comments_methods.ts"];

// Get details of the function to be inlined
const { fnParams, returnParamIndex, fnBodySource } = extractInlinedFunction(
  INLINE_FUNC_PATH,
  INLINE_FUNC_NAME,
);

/**
 * Plugin to inline calls to binary search helper function `firstTokenAtOrAfter` into its call sites.
 *
 * This eliminates function call overhead for the binary search used by all token/comment methods.
 * The function definition is read from the source file, so edits to the helper function are automatically
 * reflected in the inlined output.
 *
 * The function is inlined into all call sites in `FILES` list above.
 *
 * ```ts
 * // Original code
 * const index = firstTokenAtOrAfter(int32, rangeStart, searchFromIndex, length);
 *
 * // After transform
 * let index = searchFromIndex;
 * for (let endIndex = length; index < endIndex; ) {
 *   const mid = (index + endIndex) >> 1;
 *   if (int32[mid << 2] < rangeStart) {
 *     index = mid + 1;
 *   } else {
 *     endIndex = mid;
 *   }
 * }
 * ```
 */
const plugin: Plugin = {
  name: "inline-binary-search",
  transform: {
    filter: {
      id: new RegExp("(?:" + FILES.map((filename) => RegExp.escape(filename)).join("|") + ")$"),
    },

    handler(code, path, meta) {
      const magicString = meta.magicString!;
      const program = parse(path, code);

      // Set of call expression nodes that have been inlined
      const inlinedCallExprs = new Set<ESTree.CallExpression>();

      /**
       * Convert offset to line number.
       * @param offset - Offset in source code
       * @returns Line number
       */
      function lineNumber(offset: number): number {
        let line = 1;
        for (let i = 0; i < offset; i++) {
          if (code[i] === "\n") line++;
        }
        return line;
      }

      // Visit AST.
      // Inline call sites and check for any calls that could not be inlined.
      const visitor = new Visitor({
        VariableDeclaration(varDecl: ESTree.VariableDeclaration) {
          if (varDecl.declarations.length !== 1) return;

          const declarator = varDecl.declarations[0];
          if (declarator.id.type !== "Identifier" || !declarator.init) return;

          let declKind = varDecl.kind;
          if (!["const", "let", "var"].includes(declKind)) return;
          if (declKind === "const") declKind = "let";

          let callNode: ESTree.CallExpression;
          let suffix: string | null = null;

          const { init } = declarator;
          if (isTargetCall(init)) {
            callNode = init;
          } else if (init.type === "BinaryExpression" && isTargetCall(init.left)) {
            callNode = init.left;
            // e.g. " - 1" from `firstTokenAtOrAfter(...) - 1`
            suffix = code.slice(callNode.end, init.end);
          } else {
            return;
          }

          const args = callNode.arguments.map((arg) => {
            if (arg.type !== "Identifier" && arg.type !== "Literal") {
              throw new Error(
                `Unexpected parameter type in \`${INLINE_FUNC_NAME}\` call ` +
                  `at line ${lineNumber(arg.start)}: ${arg.type}`,
              );
            }
            return code.slice(arg.start, arg.end);
          });

          if (args.length !== fnParams.length) {
            throw new Error(
              `\`${INLINE_FUNC_NAME}\` called with ${args.length} args, expected ${fnParams.length} ` +
                `at line ${lineNumber(callNode.start)}`,
            );
          }

          // Build replacement.
          // `let <varName> = <returnParamArg>;`
          const varName = declarator.id.name;
          let replacement = `${declKind} ${varName} = ${args[returnParamIndex]};\n`;

          // Build inlined body by replacing parameter names with argument expressions.
          // The return parameter is replaced with the declared variable name.
          let inlined = fnBodySource;
          for (let i = 0; i < fnParams.length; i++) {
            const replacementVar = i === returnParamIndex ? varName : args[i];
            inlined = inlined.replace(new RegExp(`\\b${fnParams[i]}\\b`, "g"), replacementVar);
          }
          replacement += inlined;

          // If there's a suffix (e.g. ` - 1`), append `<varName> = <varName><suffix>;`
          if (suffix !== null) replacement += `\n${varName} = ${varName}${suffix};`;

          magicString.overwrite(varDecl.start, varDecl.end, replacement);

          // Record the call expression, so `CallExpression` visitor doesn't throw an error when visiting it
          inlinedCallExprs.add(callNode);
        },

        CallExpression(callExpr: ESTree.CallExpression) {
          if (isTargetCall(callExpr) && !inlinedCallExprs.has(callExpr)) {
            throw new Error(
              `\`${INLINE_FUNC_NAME}\` call on line ${lineNumber(callExpr.start)} could not be inlined. ` +
                "All calls must be in a variable declaration.",
            );
          }
        },
      });
      visitor.visit(program);

      // Check some calls were inlined. If there weren't, probably paths in `FILES` are wrong.
      if (inlinedCallExprs.size === 0) {
        throw new Error(`No \`${INLINE_FUNC_NAME}\` calls found in ${path}`);
      }

      return { code: magicString };
    },
  },
};

export default plugin;

/**
 * Check if a node is a call to the function to be inlined.
 * @param node - AST node to check
 * @returns `true` if `node` is a call to the function to be inlined
 */
function isTargetCall(node: ESTree.Node): node is ESTree.CallExpression {
  return (
    node.type === "CallExpression" &&
    node.callee.type === "Identifier" &&
    node.callee.name === INLINE_FUNC_NAME
  );
}

/**
 * Parse the function to be inlined by plugin.
 * Extracts function parameter names, the return parameter index, and the body source text.
 *
 * @param path - Path to file the function is defined in
 * @param funcName - Name of the function to find
 */
function extractInlinedFunction(
  path: string,
  funcName: string,
): { fnParams: string[]; returnParamIndex: number; fnBodySource: string } {
  const code = fs.readFileSync(path, "utf8");
  const program = parse(path, code);

  // Find the function declaration
  let funcDecl: ESTree.Function | undefined;
  for (const stmt of program.body) {
    let maybeFuncDecl: ESTree.Statement | ESTree.Declaration = stmt;
    if (stmt.type === "ExportNamedDeclaration" && stmt.declaration !== null) {
      maybeFuncDecl = stmt.declaration;
    }

    if (maybeFuncDecl.type === "FunctionDeclaration" && maybeFuncDecl.id?.name === funcName) {
      funcDecl = maybeFuncDecl;
      break;
    }
  }
  if (!funcDecl) throw new Error(`Failed to find function \`${funcName}\``);

  // Get function parameter names
  const fnParams = funcDecl.params.map((param) => {
    if (param.type !== "Identifier") {
      throw new Error(`Unexpected parameter type in \`${funcName}\`: ${param.type}`);
    }
    return param.name;
  });

  // Find return statement and the function param that matches it
  const { body } = funcDecl;
  if (body === null) throw new Error(`\`${funcName}\` has no body`);

  const lastStmt = body.body.at(-1);
  if (
    !lastStmt ||
    lastStmt.type !== "ReturnStatement" ||
    lastStmt.argument?.type !== "Identifier"
  ) {
    throw new Error(`\`${funcName}\` must end with \`return <identifier>;\``);
  }

  const returnParamName = lastStmt.argument.name;
  const returnParamIndex = fnParams.indexOf(returnParamName);
  if (returnParamIndex === -1) {
    throw new Error(`Return value \`${returnParamName}\` is not a parameter of \`${funcName}\``);
  }

  // Get function body's code to be inlined - everything from after `{` to before `return ...`)
  const fnBodySource = code.slice(body.start + 1, lastStmt.start).trim();

  return { fnParams, returnParamIndex, fnBodySource };
}
