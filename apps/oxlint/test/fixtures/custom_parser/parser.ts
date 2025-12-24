/**
 * Mock custom parser for testing custom parser support.
 *
 * This parser demonstrates the ESLint parser interface with both `parse` and `parseForESLint`.
 * It parses a simple DSL where:
 * - Lines starting with `var ` declare variables
 * - Lines starting with `log ` log expressions
 * - Lines starting with `#` are comments (ignored)
 *
 * Example input:
 * ```
 * var foo
 * log foo
 * # this is a comment
 * ```
 *
 * The parser converts this to a valid ESTree AST.
 */

interface ParserOptions {
  filePath?: string;
  ecmaVersion?: number;
}

interface SourceLocation {
  start: { line: number; column: number };
  end: { line: number; column: number };
}

interface BaseNode {
  type: string;
  start: number;
  end: number;
  range: [number, number];
  loc: SourceLocation;
}

interface Identifier extends BaseNode {
  type: "Identifier";
  name: string;
}

interface VariableDeclarator extends BaseNode {
  type: "VariableDeclarator";
  id: Identifier;
  init: null;
}

interface VariableDeclaration extends BaseNode {
  type: "VariableDeclaration";
  kind: "var";
  declarations: VariableDeclarator[];
}

interface CallExpression extends BaseNode {
  type: "CallExpression";
  callee: Identifier;
  arguments: Identifier[];
  optional: boolean;
}

interface ExpressionStatement extends BaseNode {
  type: "ExpressionStatement";
  expression: CallExpression;
}

type Statement = VariableDeclaration | ExpressionStatement;

interface Program extends BaseNode {
  type: "Program";
  body: Statement[];
  sourceType: "script" | "module";
}

/**
 * Parse the custom DSL into an ESTree AST.
 */
function parseCustomDSL(code: string, _options?: ParserOptions): Program {
  const lines = code.split("\n");
  const body: Statement[] = [];
  let offset = 0;

  for (let lineNum = 0; lineNum < lines.length; lineNum++) {
    const line = lines[lineNum];
    const lineStart = offset;
    const lineEnd = offset + line.length;
    offset = lineEnd + 1; // +1 for newline

    const trimmed = line.trim();

    // Skip empty lines and comments
    if (trimmed === "" || trimmed.startsWith("#")) {
      continue;
    }

    // Variable declaration: `var <name>`
    if (trimmed.startsWith("var ")) {
      const name = trimmed.slice(4).trim();
      if (name) {
        const varStart = line.indexOf("var");
        const nameStart = line.indexOf(name, varStart + 3);

        body.push({
          type: "VariableDeclaration",
          kind: "var",
          start: lineStart + varStart,
          end: lineEnd,
          range: [lineStart + varStart, lineEnd],
          loc: {
            start: { line: lineNum + 1, column: varStart },
            end: { line: lineNum + 1, column: line.length },
          },
          declarations: [
            {
              type: "VariableDeclarator",
              start: lineStart + nameStart,
              end: lineEnd,
              range: [lineStart + nameStart, lineEnd],
              loc: {
                start: { line: lineNum + 1, column: nameStart },
                end: { line: lineNum + 1, column: line.length },
              },
              id: {
                type: "Identifier",
                name,
                start: lineStart + nameStart,
                end: lineStart + nameStart + name.length,
                range: [lineStart + nameStart, lineStart + nameStart + name.length],
                loc: {
                  start: { line: lineNum + 1, column: nameStart },
                  end: { line: lineNum + 1, column: nameStart + name.length },
                },
              },
              init: null,
            },
          ],
        });
      }
    }

    // Log statement: `log <expr>`
    if (trimmed.startsWith("log ")) {
      const expr = trimmed.slice(4).trim();
      if (expr) {
        const logStart = line.indexOf("log");
        const exprStart = line.indexOf(expr, logStart + 3);

        body.push({
          type: "ExpressionStatement",
          start: lineStart + logStart,
          end: lineEnd,
          range: [lineStart + logStart, lineEnd],
          loc: {
            start: { line: lineNum + 1, column: logStart },
            end: { line: lineNum + 1, column: line.length },
          },
          expression: {
            type: "CallExpression",
            start: lineStart + logStart,
            end: lineEnd,
            range: [lineStart + logStart, lineEnd],
            loc: {
              start: { line: lineNum + 1, column: logStart },
              end: { line: lineNum + 1, column: line.length },
            },
            callee: {
              type: "Identifier",
              name: "console.log",
              start: lineStart + logStart,
              end: lineStart + logStart + 3,
              range: [lineStart + logStart, lineStart + logStart + 3],
              loc: {
                start: { line: lineNum + 1, column: logStart },
                end: { line: lineNum + 1, column: logStart + 3 },
              },
            },
            arguments: [
              {
                type: "Identifier",
                name: expr,
                start: lineStart + exprStart,
                end: lineStart + exprStart + expr.length,
                range: [lineStart + exprStart, lineStart + exprStart + expr.length],
                loc: {
                  start: { line: lineNum + 1, column: exprStart },
                  end: { line: lineNum + 1, column: exprStart + expr.length },
                },
              },
            ],
            optional: false,
          },
        });
      }
    }
  }

  return {
    type: "Program",
    body,
    sourceType: "script",
    start: 0,
    end: code.length,
    range: [0, code.length],
    loc: {
      start: { line: 1, column: 0 },
      end: { line: lines.length, column: lines[lines.length - 1]?.length ?? 0 },
    },
  };
}

/**
 * Simple parse function - returns just the AST.
 */
export function parse(code: string, options?: ParserOptions): Program {
  return parseCustomDSL(code, options);
}

/**
 * parseForESLint function - returns AST and additional metadata.
 * This is the preferred method when available.
 */
export function parseForESLint(
  code: string,
  options?: ParserOptions,
): {
  ast: Program;
  scopeManager?: unknown;
  visitorKeys?: Record<string, string[]>;
  services?: Record<string, unknown>;
} {
  const ast = parseCustomDSL(code, options);

  return {
    ast,
    // For Phase 1, we don't provide scope info
    scopeManager: undefined,
    // Standard ESTree visitor keys
    visitorKeys: {
      Program: ["body"],
      VariableDeclaration: ["declarations"],
      VariableDeclarator: ["id", "init"],
      ExpressionStatement: ["expression"],
      CallExpression: ["callee", "arguments"],
      Identifier: [],
    },
    // Custom services that rules can access via context.parserServices
    services: {
      customParserUsed: true,
      parserName: "custom-dsl-parser",
    },
  };
}
