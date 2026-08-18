import { describe, expect, it } from "vitest";

import { transform, transformSync } from "../index";

const fixture = `// @license MIT
import { useState } from "react";

interface Props {
  text: string;
}

export function Component(props: Props) {
  const [count, setCount] = useState<number>(0);
  return <button onClick={() => setCount(count + 1)}>{props.text}: {count}</button>;
}
`;

describe("transformSync", () => {
  it("runs React Compiler before TypeScript and JSX transforms", () => {
    const result = transformSync("Component.tsx", fixture);

    expect(result.fatal).toBe(false);
    expect(result.errors).toEqual([]);
    expect(result.code).toContain("react/compiler-runtime");
    expect(result.code).toContain("_c(");
    expect(result.code).not.toContain("interface Props");
    expect(result.code).not.toContain("<button");
    expect(result.code).toContain("@license MIT");
  });

  it("honors JSX pragmas after React Compiler adds imports", () => {
    const result = transformSync(
      "Component.tsx",
      `/** @jsxRuntime automatic */
/** @jsxImportSource custom-runtime */
export function Component({ value }: { value: string }) {
  return <div>{value}</div>;
}
`,
    );

    expect(result.fatal).toBe(false);
    expect(result.errors).toEqual([]);
    expect(result.code).toContain("react/compiler-runtime");
    expect(result.code).toContain('from "custom-runtime/jsx-runtime"');
    expect(result.code).not.toContain('from "react/jsx-runtime"');
  });

  it("forwards React Compiler options", () => {
    const target = transformSync("Component.tsx", fixture, {
      reactCompiler: { target: "18" },
    });
    expect(target.errors).toEqual([]);
    expect(target.code).toContain("react-compiler-runtime");

    const gated = transformSync("Component.tsx", fixture, {
      reactCompiler: {
        gating: {
          source: "feature-flags",
          importSpecifierName: "isCompilerEnabled",
        },
      },
    });
    expect(gated.errors).toEqual([]);
    expect(gated.code).toContain("feature-flags");
    expect(gated.code).toContain("isCompilerEnabled");
  });

  it("supports dynamic gating and Meta-internal targets", () => {
    const dynamic = transformSync(
      "Component.jsx",
      `export function Component(props) {
        "use memo if(isCompilerEnabled)";
        return <div>{props.text}</div>;
      }`,
      {
        reactCompiler: {
          dynamicGating: { source: "dynamic-feature-flags" },
        },
      },
    );
    expect(dynamic.errors).toEqual([]);
    expect(dynamic.code).toContain("dynamic-feature-flags");
    expect(dynamic.code).toContain("isCompilerEnabled");

    const meta = transformSync("Component.tsx", fixture, {
      reactCompiler: {
        target: {
          kind: "donotuse_meta_internal",
          runtimeModule: "custom-react-runtime",
        },
      },
    });
    expect(meta.errors).toEqual([]);
    expect(meta.code).toContain("custom-react-runtime");
  });

  it("can override opt-out directives", () => {
    const source = `function Component(props) {
      "use no memo";
      return <div>{props.text}</div>;
    }`;

    const optedOut = transformSync("Component.jsx", source);
    expect(optedOut.errors).toEqual([]);
    expect(optedOut.code).not.toContain("_c(");

    const compiled = transformSync("Component.jsx", source, {
      reactCompiler: { ignoreUseNoForget: true },
    });
    expect(compiled.errors).toEqual([]);
    expect(compiled.code).toContain("_c(");
  });

  it.each([
    ["compilationMode", { compilationMode: "bogus" }],
    ["panicThreshold", { panicThreshold: "bogus" }],
    ["outputMode", { outputMode: "bogus" }],
    ["target", { target: "20" }],
    [
      "environment.validateExhaustiveEffectDependencies",
      {
        environment: {
          validateExhaustiveEffectDependencies: "bogus",
        },
      },
    ],
  ])("reports an invalid %s option without emitting code", (option, options) => {
    const result = transformSync("Component.tsx", fixture, {
      reactCompiler: options,
    } as never);
    expect(result.fatal).toBe(true);
    expect(result.code).toBe("");
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0].message).toContain(`Invalid React Compiler \`${option}\` option`);
  });

  it("marks parse errors as fatal", () => {
    const result = transformSync("Component.tsx", "function Component(");

    expect(result.fatal).toBe(true);
    expect(result.code).toBe("");
    expect(result.errors.length).toBeGreaterThan(0);
  });

  it("supports source maps and language overrides", () => {
    const result = transformSync("Component.vue", fixture, {
      lang: "tsx",
      sourcemap: true,
    });

    expect(result.errors).toEqual([]);
    expect(result.map).toMatchObject({
      sources: ["Component.vue"],
      sourcesContent: [fixture],
      version: 3,
    });
  });

  it("can filter files with sources", () => {
    const result = transformSync("vendor/Component.tsx", fixture, {
      reactCompiler: { sources: ["src/"] },
    });

    expect(result.errors).toEqual([]);
    expect(result.code).not.toContain("react/compiler-runtime");
    expect(result.code).not.toContain("interface Props");
    expect(result.code).not.toContain("<button");
  });

  it("skips node_modules by default", () => {
    for (const options of [undefined, { reactCompiler: {} }]) {
      const result = transformSync("node_modules/package/Component.tsx", fixture, options);

      expect(result.errors).toEqual([]);
      expect(result.code).not.toContain("react/compiler-runtime");
      expect(result.code).not.toContain("_c(");
    }
  });

  it("allows sources to include node_modules", () => {
    const result = transformSync("node_modules/package/Component.tsx", fixture, {
      reactCompiler: { sources: ["node_modules/package"] },
    });

    expect(result.errors).toEqual([]);
    expect(result.code).toContain("react/compiler-runtime");
    expect(result.code).toContain("_c(");
  });

  it("keeps imports used by compiled computed keys", () => {
    const result = transformSync(
      "Box.tsx",
      `import { CSS_VAR } from "./styles.css";
      export function Box({ size }) {
        const style = { [CSS_VAR]: size + "px" };
        return <div style={style} />;
      }`,
    );

    expect(result.errors).toEqual([]);
    expect(result.code).toContain("_c(");
    expect(result.code).toContain("[CSS_VAR]");
    expect(result.code).toContain("import { CSS_VAR }");
  });

  it("honors ESLint suppressions when internal validations are enabled", () => {
    const result = transformSync(
      "Counter.jsx",
      `import { useEffect, useRef, useState } from "react";
      export function Counter({ step }) {
        const [count, setCount] = useState(0);
        const ref = useRef(step);
        useEffect(() => {
          setCount((value) => value + ref.current);
          // eslint-disable-next-line react-hooks/exhaustive-deps
        }, []);
        return <div>{count}</div>;
      }`,
      {
        reactCompiler: {
          environment: {
            validateExhaustiveMemoizationDependencies: true,
          },
        },
      },
    );

    expect(result.fatal).toBe(false);
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0]).toMatchObject({
      severity: "Warning",
      message: "React rule suppression prevents optimization",
    });
    expect(result.code).not.toContain("react/compiler-runtime");
    expect(result.code).not.toContain("_c(");
  });

  it("honors ESLint suppressions by default", () => {
    const result = transformSync(
      "Component.tsx",
      `function Component({ value }: { value: number }) {
        // eslint-disable-next-line react-hooks/exhaustive-deps
        const doubled = value * 2;
        return <div>{doubled}</div>;
      }`,
    );

    expect(result.fatal).toBe(false);
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0].severity).toBe("Warning");
    expect(result.errors[0].message).toBe("React rule suppression prevents optimization");
    expect(result.code).not.toContain("react/compiler-runtime");
  });

  it("allows ESLint suppression bailouts to be disabled", () => {
    const result = transformSync(
      "Component.tsx",
      `function Component({ value }: { value: number }) {
        // eslint-disable-next-line react-hooks/exhaustive-deps
        const doubled: number = value * 2;
        return <div>{doubled}</div>;
      }`,
      {
        reactCompiler: {
          eslintSuppressionRules: [],
        },
      },
    );

    expect(result.fatal).toBe(false);
    expect(result.errors).toEqual([]);
    expect(result.code).toContain("react/compiler-runtime");
    expect(result.code).toContain("_c(");
    expect(result.code).not.toContain(": number");
    expect(result.code).not.toContain("<div");
  });

  it("continues after an incompatible library bailout", () => {
    const result = transformSync(
      "Components.tsx",
      `import { useReactTable } from "@tanstack/react-table";
      function Table() {
        const table = useReactTable({});
        return <div>{table}</div>;
      }
      export function Component(props: { text: string }) {
        return <span>{props.text}</span>;
      }`,
    );

    expect(result.fatal).toBe(false);
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0]).toMatchObject({
      severity: "Warning",
      message: "Use of incompatible library",
    });
    expect(result.errors.some((error) => error.message.includes("Unexpected error"))).toBe(false);
    expect(result.code).toContain("react/compiler-runtime");
    expect(result.code).not.toContain("props: { text: string }");
    expect(result.code).not.toContain("<span");
  });

  it("does not enable manual memo dependency validation by default", () => {
    const result = transformSync(
      "Component.tsx",
      `import { useMemo } from "react";
      import { typedCapture, typedCreateFrom, typedMutate, ValidateMemoization } from "shared-runtime";
      function Component({ a, b }: { a: number; b: number }) {
        const x = useMemo(() => ({ a }), [a, b]);
        const y = typedCapture(x);
        const z = typedCreateFrom(y);
        typedMutate(z, b);
        return <ValidateMemoization inputs={[a, b]} output={x} />;
      }`,
    );

    expect(result.errors).toEqual([]);
    expect(result.code).toContain("react/compiler-runtime");
  });

  it("compiles an unsuppressed sibling after a suppression bailout", () => {
    const result = transformSync(
      "Components.tsx",
      `function Suppressed({ value }: { value: number }) {
        // eslint-disable-next-line react-hooks/exhaustive-deps
        const doubled = value * 2;
        return <div>{doubled}</div>;
      }
      export function Component(props: { text: string }) {
        return <span>{props.text}</span>;
      }`,
    );

    expect(result.fatal).toBe(false);
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0].severity).toBe("Warning");
    expect(result.errors[0].message).toBe("React rule suppression prevents optimization");
    expect(result.code).toContain("react/compiler-runtime");
    expect(result.code).not.toContain("props: { text: string }");
    expect(result.code).not.toContain("<span");
  });

  it.each(["critical_errors", "all_errors"] as const)(
    "makes suppression bailouts fatal at panicThreshold %s",
    (panicThreshold) => {
      const result = transformSync(
        "Component.tsx",
        `function Component({ value }: { value: number }) {
          // eslint-disable-next-line react-hooks/exhaustive-deps
          const doubled = value * 2;
          return <div>{doubled}</div>;
        }`,
        {
          reactCompiler: {
            panicThreshold,
          },
        },
      );

      expect(result.fatal).toBe(true);
      expect(result.code).toBe("");
      expect(result.errors).toHaveLength(1);
      expect(result.errors[0].severity).toBe("Error");
      expect(result.errors[0].message).toBe("React rule suppression prevents optimization");
    },
  );

  it("makes warning bailouts fatal at panicThreshold all_errors", () => {
    const result = transformSync(
      "Table.tsx",
      `import { useReactTable } from "@tanstack/react-table";
      function Table() {
        const table = useReactTable({});
        return <div>{table}</div>;
      }`,
      { reactCompiler: { panicThreshold: "all_errors" } },
    );

    expect(result.fatal).toBe(true);
    expect(result.code).toBe("");
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0]).toMatchObject({
      severity: "Warning",
      message: "Use of incompatible library",
    });
  });

  it("can disable and explicitly enable React Compiler", () => {
    const disabled = transformSync("Component.tsx", fixture, {
      reactCompiler: false,
    });
    expect(disabled.errors).toEqual([]);
    expect(disabled.code).not.toContain("react/compiler-runtime");
    expect(disabled.code).not.toContain("interface Props");
    expect(disabled.code).not.toContain("<button");

    const enabled = transformSync("Component.tsx", fixture, {
      reactCompiler: true,
    });
    expect(enabled.errors).toEqual([]);
    expect(enabled.code).toContain("react/compiler-runtime");
  });

  it("configures and preserves JSX independently of React Compiler", () => {
    const configured = transformSync("Component.tsx", fixture, {
      reactCompiler: false,
      jsx: { importSource: "custom-jsx" },
    });
    expect(configured.errors).toEqual([]);
    expect(configured.code).toContain('from "custom-jsx/jsx-runtime"');
    expect(configured.code).not.toContain("<button");

    const preserved = transformSync("Component.tsx", fixture, {
      jsx: "preserve",
    });
    expect(preserved.errors).toEqual([]);
    expect(preserved.code).toContain("react/compiler-runtime");
    expect(preserved.code).not.toContain("interface Props");
    expect(preserved.code).toContain("<button");
  });

  it("reports invalid JSX modes without emitting code", () => {
    const result = transformSync("Component.tsx", fixture, {
      reactCompiler: false,
      jsx: "invalid",
    } as never);
    expect(result.fatal).toBe(true);
    expect(result.code).toBe("");
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0].message).toContain("Invalid `jsx` option");
  });

  it("supports React Fast Refresh through JSX options", () => {
    const result = transformSync("Component.tsx", fixture, {
      jsx: { refresh: { emitFullSignatures: true } },
    });
    expect(result.errors).toEqual([]);
    expect(result.code).toContain("react/compiler-runtime");
    expect(result.code).toContain("$RefreshSig$");
    expect(result.code).toContain("$RefreshReg$");
    expect(result.code).toContain("useState{[count, setCount](0)}");
  });
});

describe("transform", () => {
  it("matches the synchronous transform", async () => {
    const sync = transformSync("Component.tsx", fixture);
    const asyncResult = await transform("Component.tsx", fixture);

    expect(asyncResult).toEqual(sync);
  });
});
