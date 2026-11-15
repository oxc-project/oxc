import { basename } from "node:path";
import { parse } from "@babel/parser";
import type { types } from "@babel/core";

import { runtimeExclusions } from "./exclusions.ts";

export function exclusion(path: string, source: string): string | null {
  if (/^(error\.|todo\.error)/.test(basename(path))) return "Expected compiler diagnostics";
  const header = source.split("\n", 1)[0]!;
  if (header.includes('@outputMode:"lint"')) return "Lint-only fixture";
  if (header.includes("@flow")) return "Flow is unsupported by the Oxc parser";
  const reason = runtimeExclusions.get(path);
  if (reason !== undefined) return reason;
  const ast = parse(source, {
    sourceType: header.includes("@script") ? "script" : "module",
    plugins: ["typescript", "jsx"],
  });
  function check(value: types.Node | null | undefined): null {
    if (
      value?.type === "ObjectExpression"
      && value.properties.some(
        (property) =>
          property.type === "ObjectProperty"
          && property.value.type === "StringLiteral"
          && property.value.value === "TodoAdd",
      )
    ) {
      throw new Error(
        `${path}: replace the TodoAdd FIXTURE_ENTRYPOINT placeholder with real inputs`,
      );
    }
    return null;
  }
  for (const statement of ast.program.body) {
    if (
      statement.type === "ExpressionStatement"
      && statement.expression.type === "AssignmentExpression"
      && statement.expression.operator === "="
      && statement.expression.left.type === "Identifier"
      && statement.expression.left.name === "exports"
      && statement.expression.right.type === "ObjectExpression"
    ) {
      const entrypoint = statement.expression.right.properties.find(
        (property) =>
          property.type === "ObjectProperty"
          && property.key.type === "Identifier"
          && property.key.name === "FIXTURE_ENTRYPOINT",
      );
      if (entrypoint?.type === "ObjectProperty") {
        return check(entrypoint.value);
      }
    }
    if (
      statement.type !== "ExportNamedDeclaration"
      || statement.declaration?.type !== "VariableDeclaration"
    ) {
      continue;
    }
    for (const declaration of statement.declaration.declarations) {
      if (declaration.id.type !== "Identifier" || declaration.id.name !== "FIXTURE_ENTRYPOINT") {
        continue;
      }
      return check(declaration.init);
    }
  }
  throw new Error(
    `${path}: export a usable FIXTURE_ENTRYPOINT with real inputs, or document a concrete runtime limitation in tasks/sprout/src/exclusions.ts`,
  );
}
