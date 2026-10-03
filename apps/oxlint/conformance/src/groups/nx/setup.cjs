// Use Node's loader for Oxlint's ESM bundle and native bindings, outside Jest's
// module registry. Nx's filesystem mocks must only affect the rule and fixtures.
const nativeRequire = process.getBuiltinModule("module").createRequire(__filename);
const fs = process.getBuiltinModule("fs");
const calls = [];
const utilsPath = nativeRequire.resolve("@typescript-eslint/utils", { paths: [process.cwd()] });
const parserPath = nativeRequire.resolve("@typescript-eslint/parser", { paths: [process.cwd()] });

if (process.env.NX_CONFORMANCE_OXLINT) {
  const { RuleTester } = nativeRequire(process.env.NX_CONFORMANCE_OXLINT);
  // Implement only the single-rule flat config used by this pinned Nx suite.
  class Linter {
    verify(code, configs, filename) {
      if (configs.length !== 1) throw new Error("Expected one flat config");
      const config = configs[0];
      const entries = Object.entries(config.rules);
      if (entries.length !== 1) throw new Error("Expected one rule");
      const [ruleId, [severity, ...options]] = entries[0];
      const separator = ruleId.lastIndexOf("/");
      const pluginName = ruleId.slice(0, separator);
      const ruleName = ruleId.slice(separator + 1);
      const plugin = config.plugins[pluginName];
      const { parser, ...languageOptions } = config.languageOptions;
      if (parser.parseForESLint !== require(parserPath).parseForESLint) {
        throw new Error("Only Nx's TypeScript parser is supported");
      }
      const test = { code, filename, options, languageOptions, eslintCompat: true };
      calls.push(test);
      return RuleTester.lintForConformance(test, {
        meta: { name: pluginName },
        rules: { [ruleName]: plugin.rules[ruleName] },
      }).map(({ fixes, ...diagnostic }) => ({
        ...diagnostic,
        severity: severity === "error" ? 2 : 1,
        ...(fixes === null ? {} : { fix: mergeFixes(code, fixes) }),
      }));
    }
  }
  jest.mock(utilsPath, () => {
    const actual = jest.requireActual(utilsPath);
    return { ...actual, TSESLint: { ...actual.TSESLint, Linter } };
  });
}

// Keep original source for reports even when an upstream assertion fails.
afterEach(() => {
  if (process.env.NX_CONFORMANCE_CASES) {
    fs.appendFileSync(
      process.env.NX_CONFORMANCE_CASES,
      JSON.stringify({
        name: expect.getState().currentTestName,
        calls: calls.splice(0),
      }) + "\n",
    );
  }
});

function mergeFixes(code, fixes) {
  const sorted = [...fixes].sort((a, b) => a.start - b.start);
  const start = sorted[0].start;
  let end = start;
  let text = "";
  for (const fix of sorted) {
    if (fix.start < end) throw new Error("Overlapping fixes");
    text += code.slice(end, fix.start) + fix.text;
    end = fix.end;
  }
  return { range: [start, end], text };
}
