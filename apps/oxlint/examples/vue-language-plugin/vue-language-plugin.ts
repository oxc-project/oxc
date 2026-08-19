import { defineLanguagePlugin } from "#oxlint/language-plugins";
import type {
  LanguageNode,
  LanguageParseResult,
  Mapping,
  TransformResult,
} from "#oxlint/language-plugins";

/**
 * Minimal Vue SFC language plugin (RFC #21936 example).
 *
 * Real implementations should use a proper Vue parser (e.g. vue-eslint-parser /
 * @vue/compiler-sfc) and a Volar-faithful virtual code generator. This example
 * only sketches the shape of the API.
 */
export default defineLanguagePlugin({
  meta: { name: "vue-language-plugin" },
  // Prefer extension / filename defaults (not unrestricted globs).
  defaultFiles: [".vue"],
  visitorKeys: {
    nodes: {
      Program: { body: ["Statement"] },
      VElement: {
        startTag: "VStartTag",
        children: ["VNode"],
        endTag: "VEndTag",
      },
      VStartTag: { attributes: ["VAttribute"] },
      VEndTag: {},
      VAttribute: {
        key: "VIdentifier",
        value: "VAttributeValue",
      },
      VIdentifier: {},
      VAttributeValue: {},
      VText: {},
      VExpression: { expression: ["Expression"] },
      ExpressionStatement: { expression: ["Expression"] },
      VariableDeclaration: { declarations: ["VariableDeclarator"] },
      VariableDeclarator: { id: "Identifier", init: ["Expression"] },
      Identifier: {},
      CallExpression: { callee: ["Expression"], arguments: ["Expression"] },
      MemberExpression: { object: ["Expression"], property: ["Expression"] },
      UpdateExpression: { argument: ["Expression"] },
      FunctionDeclaration: { id: ["Identifier"], body: "BlockStatement" },
      BlockStatement: { body: ["Statement"] },
      ImportDeclaration: { specifiers: ["ImportSpecifier"], source: "Literal" },
      ImportSpecifier: { imported: "Identifier", local: "Identifier" },
      Literal: {},
    },
    unions: {
      VNode: ["VElement", "VText", "VExpression"],
      Statement: [
        "ExpressionStatement",
        "VariableDeclaration",
        "FunctionDeclaration",
        "ImportDeclaration",
      ],
      Expression: [
        "Identifier",
        "CallExpression",
        "MemberExpression",
        "UpdateExpression",
        "Literal",
      ],
    },
  },
  parse(filePath, sourceText, _options) {
    return parseVueSfc(filePath, sourceText);
  },
  load(_filePath, parseResult, _sourceText, _options) {
    return {
      languageId: "vue",
      ast: parseResult.ast,
      tokens: parseResult.tokens,
      transform: parseResult.transform ?? null,
      isESTree: false,
      parserServices: {
        // Placeholder for defineProps / template bindings / etc.
        vue: { kind: "sfc" },
      },
    };
  },
});

function parseVueSfc(filePath: string, sourceText: string): LanguageParseResult {
  const scriptMatch = sourceText.match(/<script\b([^>]*)>([\s\S]*?)<\/script>/i);
  const templateMatch = sourceText.match(/<template\b([^>]*)>([\s\S]*?)<\/template>/i);

  const scriptOffset = scriptMatch
    ? sourceText.indexOf(scriptMatch[0]) + scriptMatch[0].indexOf(scriptMatch[2])
    : 0;
  const scriptSource = scriptMatch?.[2] ?? "";
  const templateSource = templateMatch?.[2] ?? "";

  const body: LanguageNode[] = [];

  // Extremely small script "AST" — only enough for the RFC example file.
  for (const line of scriptSource.split("\n")) {
    const trimmed = line.trim();
    if (trimmed.startsWith("import ")) {
      body.push({
        type: "ImportDeclaration",
        source: { type: "Literal", value: "vue" },
        specifiers: [],
      });
    } else if (trimmed.startsWith("const ")) {
      body.push({
        type: "VariableDeclaration",
        kind: "const",
        declarations: [
          {
            type: "VariableDeclarator",
            id: { type: "Identifier", name: "count" },
            init: {
              type: "CallExpression",
              callee: { type: "Identifier", name: "ref" },
              arguments: [{ type: "Literal", value: 0 }],
            },
          },
        ],
      });
    } else if (trimmed.startsWith("function ")) {
      body.push({
        type: "FunctionDeclaration",
        id: { type: "Identifier", name: "inc" },
        body: { type: "BlockStatement", body: [] },
      });
    }
  }

  const templateChildren: LanguageNode[] = [];
  const buttonMatch = templateSource.match(/<button\b([^>]*)>([\s\S]*?)<\/button>/i);
  if (buttonMatch) {
    templateChildren.push({
      type: "VElement",
      tag: "button",
      startTag: { type: "VStartTag", attributes: [] },
      children: [
        {
          type: "VExpression",
          expression: { type: "Identifier", name: "count" },
        },
      ],
      endTag: { type: "VEndTag" },
    });
  }

  // Surface template as a top-level VElement so JS rules can visit it.
  if (templateChildren.length > 0 || templateSource.trim()) {
    body.push({
      type: "VElement",
      tag: "template",
      startTag: { type: "VStartTag", attributes: [] },
      children:
        templateChildren.length > 0
          ? templateChildren
          : [{ type: "VText", value: templateSource }],
      endTag: { type: "VEndTag" },
    });
  }

  const ast: LanguageNode = {
    type: "Program",
    body,
    sourceType: "module",
  };

  const transform = buildExampleTransform(filePath, scriptSource, scriptOffset);

  return { ast, tokens: [], transform };
}

/**
 * Build a semantically-oriented virtual TS file for the RFC example.
 *
 * Production plugins should emit Volar-faithful virtual code (preserving macros),
 * not Vue compiler runtime output.
 */
function buildExampleTransform(
  filePath: string,
  scriptSource: string,
  scriptOffset: number,
): TransformResult {
  const componentName = filePath.split("/").pop()?.replace(/\.vue$/i, "") ?? "Component";
  const { imports, setupBody, importOriginalStart, bodyOriginalStart } = splitScriptSetup(
    scriptSource,
    scriptOffset,
  );

  // Imports must stay at module top-level — nesting them inside `setup()` is invalid ESM.
  const runtimeImport = `import { defineComponent as _defineComponent } from "vue";\n`;
  const importBlock = runtimeImport + (imports.length > 0 ? `${imports}\n` : "");
  const prefix =
    importBlock +
    `export default /*@__PURE__*/_defineComponent({\n` +
    `  __name: ${JSON.stringify(componentName)},\n` +
    `  setup(__props) {\n`;
  const suffix = `\n    return () => null;\n  },\n});\n`;
  const indentedBody = indent(setupBody, 4);
  const sourceText = `${prefix}${indentedBody}${suffix}`;

  const mappings: Mapping[] = [];
  if (imports.length > 0) {
    mappings.push({
      virtualStart: runtimeImport.length,
      virtualEnd: runtimeImport.length + imports.length,
      originalStart: importOriginalStart,
      originalEnd: importOriginalStart + imports.length,
    });
  }
  if (setupBody.length > 0) {
    mappings.push({
      virtualStart: prefix.length,
      virtualEnd: prefix.length + indentedBody.length,
      originalStart: bodyOriginalStart,
      originalEnd: bodyOriginalStart + setupBody.length,
    });
  }

  return {
    sourceText,
    scriptKind: "ts",
    mappings,
  };
}

/**
 * Split `<script setup>` source into top-level import statements vs setup body.
 */
function splitScriptSetup(
  scriptSource: string,
  scriptOffset: number,
): {
  imports: string;
  setupBody: string;
  importOriginalStart: number;
  bodyOriginalStart: number;
} {
  const lines = scriptSource.split("\n");
  const importLines: string[] = [];
  const bodyLines: string[] = [];
  let seenBody = false;

  for (const line of lines) {
    const trimmed = line.trim();
    // Skip leading blanks; keep blanks that sit inside the import block.
    if (!seenBody && trimmed.length === 0) {
      if (importLines.length > 0) importLines.push(line);
      continue;
    }
    if (!seenBody && trimmed.startsWith("import ")) {
      importLines.push(line);
      continue;
    }
    seenBody = true;
    bodyLines.push(line);
  }

  const imports = importLines.join("\n").trimEnd();
  const setupBody = bodyLines.join("\n").trim();

  let importOriginalStart = scriptOffset;
  let bodyOriginalStart = scriptOffset;
  const firstImport = importLines.find((l) => l.trim().startsWith("import "));
  if (firstImport) {
    const idx = scriptSource.indexOf(firstImport);
    if (idx >= 0) importOriginalStart = scriptOffset + idx;
  }
  const firstBody = bodyLines.find((l) => l.trim().length > 0);
  if (firstBody) {
    const idx = scriptSource.indexOf(firstBody);
    if (idx >= 0) bodyOriginalStart = scriptOffset + idx;
  }

  return {
    imports,
    setupBody,
    importOriginalStart,
    bodyOriginalStart,
  };
}


function indent(text: string, spaces: number): string {
  const pad = " ".repeat(spaces);
  return text
    .split("\n")
    .map((line) => (line.length === 0 ? line : pad + line))
    .join("\n");
}
