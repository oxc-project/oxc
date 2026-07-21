import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

import { transformSync } from "../index";

// A single fixture exercising every concern the React Compiler integration has to
// handle together: a memoizable component using a hook, TypeScript types, JSX, ES
// module syntax, and top-level comments. The compiler runs first on the pristine
// AST, then the rest of the transform pipeline (TypeScript, JSX) runs on its
// output, and codegen emits the result.
const fixture = `// @license MIT
import { useState } from "react";

interface Props {
  text: string;
  onClick: () => void;
}

// Memoized component: exercises hooks, TS types, JSX and comments.
export function Component(props: Props) {
  const [count, setCount] = useState<number>(0);
  return (
    <div onClick={() => props.onClick()}>
      {props.text}: {count}
    </div>
  );
}
`;

const nonfatalBailoutFixture = readFileSync(
  new URL("./fixtures/react-compiler/nonfatal-bailout.tsx", import.meta.url),
  "utf8",
);

describe("plugins.reactCompiler", () => {
  it("memoizes, composes with the TS + JSX transforms, and preserves comments", () => {
    const { code, errors } = transformSync("Component.tsx", fixture, {
      plugins: { reactCompiler: true },
      jsx: { runtime: "automatic" },
    });

    expect(errors).toEqual([]);

    // React Compiler memoized the component.
    expect(code).toContain("react/compiler-runtime");
    expect(code).toContain("_c(");

    // JSX was lowered via the automatic runtime — no raw JSX remains.
    expect(code).toContain("jsx");
    expect(code).not.toContain("<div");

    // TypeScript was stripped: the interface, annotations and generic are gone.
    expect(code).not.toContain("interface Props");
    expect(code).not.toContain(": Props");
    expect(code).not.toContain("<number>");

    // The hook call and ES module syntax survive.
    expect(code).toContain("useState(");
    expect(code).toContain("export function Component");

    // Top-level comments survive react_compiler -> transformer -> codegen.
    expect(code).toContain("@license MIT");
    expect(code).toContain("Memoized component");
  });

  it("accepts a ReactCompilerOptions object", () => {
    const { code } = transformSync("Component.tsx", fixture, {
      plugins: { reactCompiler: { compilationMode: "all" } },
    });
    expect(code).toContain("react/compiler-runtime");
    expect(code).toContain("_c(");
  });

  // It sits under `plugins` for JS callers, but the compiler still runs as its own pass
  // before the rest of `plugins` — so it composes with its neighbours there.
  it("composes with the other plugins it is grouped with", () => {
    const { code, errors } = transformSync(
      "Component.tsx",
      `import styled from "styled-components";
const Box = styled.div\`color: red;\`;
export function Component() {
  const [n] = useState(0);
  return <Box>{n}</Box>;
}
`,
      {
        plugins: {
          reactCompiler: true,
          styledComponents: { displayName: true },
        },
        jsx: { runtime: "automatic" },
      },
    );

    expect(errors).toEqual([]);
    // React Compiler memoized, and styled-components still got its displayName.
    expect(code).toContain("_c(");
    expect(code).toContain("displayName");
  });

  // The `ts_type` annotations constrain the string options at the type level only, so
  // a plain-JS caller can still reach the binding with an unknown value.
  it.each([
    ["compilationMode", { compilationMode: "bogus" }],
    ["panicThreshold", { panicThreshold: "bogus" }],
    ["outputMode", { outputMode: "bogus" }],
    // An unrecognized target would otherwise fall back to the React 19 runtime silently.
    ["target", { target: "20" }],
  ])("rejects an unknown `%s` value rather than ignoring it", (option, reactCompiler) => {
    const { code, errors } = transformSync("Component.tsx", fixture, {
      plugins: { reactCompiler: reactCompiler as never },
    });

    expect(code).toBe("");
    expect(errors).toHaveLength(1);
    expect(errors[0].message).toContain(`Invalid plugins.reactCompiler.${option} option:`);
  });

  // Each option below changes observable output, proving it is forwarded to the compiler.

  it("forwards `target` — 17/18 import the standalone runtime package", () => {
    const { code } = transformSync("Component.tsx", fixture, {
      plugins: { reactCompiler: { target: "18" } },
      jsx: { runtime: "automatic" },
    });
    expect(code).toContain("react-compiler-runtime");
    expect(code).not.toContain("react/compiler-runtime");
  });

  it("forwards `gating` — emits a feature-gated component", () => {
    const { code } = transformSync("Component.tsx", fixture, {
      plugins: {
        reactCompiler: {
          gating: { source: "my-gating-module", importSpecifierName: "isForgetEnabled" },
        },
      },
      jsx: { runtime: "automatic" },
    });
    expect(code).toContain("my-gating-module");
    expect(code).toContain("isForgetEnabled");
  });

  it("forwards `ignoreUseNoForget` — compiles a `use no memo` function", () => {
    const source = `function Component(props) {
  "use no memo";
  return <div>{props.text}</div>;
}
`;
    const optedOut = transformSync("Component.jsx", source, {
      plugins: { reactCompiler: true },
      jsx: { runtime: "automatic" },
    });
    expect(optedOut.code).not.toContain("_c(");

    const overridden = transformSync("Component.jsx", source, {
      plugins: { reactCompiler: { ignoreUseNoForget: true } },
      jsx: { runtime: "automatic" },
    });
    expect(overridden.code).toContain("_c(");
  });

  it("emits code when React Compiler reports warnings", () => {
    const { code, errors } = transformSync(
      "Component.jsx",
      `
function Component() {
  const fbt = "span";
  return <fbt desc="label">Hello</fbt>;
}
`,
      {
        plugins: { reactCompiler: true },
        jsx: { runtime: "automatic" },
      },
    );

    expect(code).not.toBe("");
    expect(code).toContain("function Component");
    expect(errors.some((error) => error.severity === "Warning")).toBe(true);
    expect(errors.some((error) => error.severity === "Error")).toBe(false);
  });

  it("emits fully lowered code for a default nonfatal bailout", () => {
    const { code, errors } = transformSync("Component.tsx", nonfatalBailoutFixture, {
      plugins: { reactCompiler: true },
      jsx: { runtime: "automatic" },
    });

    // Upstream logs recoverable compiler errors at their original severity but
    // does not throw with the default panicThreshold: "none".
    expect(errors.some((error) => error.severity === "Error")).toBe(true);
    expect(code).toContain("react/compiler-runtime");
    expect(code).toContain("_c(");
    expect(code).not.toContain("type Props");
    expect(code).not.toContain(": JSX.Element");
    expect(code).not.toContain("<main>");
  });

  it("aborts the transform when the panic threshold escalates an error", () => {
    const { code, errors } = transformSync("Component.tsx", nonfatalBailoutFixture, {
      plugins: { reactCompiler: { panicThreshold: "all_errors" } },
      jsx: { runtime: "automatic" },
    });

    expect(errors.some((error) => error.severity === "Error")).toBe(true);
    expect(code).toBe("");
  });

  it("keeps enum values available for the downstream TypeScript transform", () => {
    const { code, errors } = transformSync(
      "Component.tsx",
      `
enum E {
  A = 1,
  B = A + 1,
}

function Component() {
  return <div>{E.B}</div>;
}
`,
      {
        plugins: { reactCompiler: true },
        jsx: { runtime: "automatic" },
      },
    );

    expect(errors).toEqual([]);
    expect(code).toContain('E[E["B"] = 2] = "B"');
  });

  it("does nothing when omitted (the default), or when `plugins` or the option is absent/false", () => {
    for (const options of [{}, { plugins: {} }, { plugins: { reactCompiler: false } }]) {
      const { code } = transformSync("Component.tsx", fixture, options);
      expect(code).not.toContain("react/compiler-runtime");
      expect(code).not.toContain("_c(");
    }
  });
});

// `environment` mirrors the option upstream's Babel plugin leads with. Several passes are
// off by default, so without it these diagnostics are unreachable from JS.
describe("plugins.reactCompiler.environment", () => {
  const setStateInEffect = `import { useEffect, useState } from "react";
export function Component() {
  const [x, setX] = useState(0);
  useEffect(() => { setX(1); }, []);
  return <div>{x}</div>;
}
`;

  // This pass needs both the flag and `outputMode: 'lint'` — the pipeline gates it on
  // `env.config.validate_no_set_state_in_effects && env.output_mode == Lint`.
  it("reaches a validation that is off by default", () => {
    const off = transformSync("Component.tsx", setStateInEffect, {
      plugins: { reactCompiler: { outputMode: "lint" } },
      jsx: { runtime: "automatic" },
    });
    // Flag defaults to false, so the pass never runs and reports nothing.
    expect(off.errors.some((e) => /setState|effect/i.test(e.message))).toBe(false);

    const on = transformSync("Component.tsx", setStateInEffect, {
      plugins: {
        reactCompiler: {
          outputMode: "lint",
          environment: { validateNoSetStateInEffects: true },
        },
      },
      jsx: { runtime: "automatic" },
    });
    expect(on.errors.length).toBeGreaterThan(0);
    expect(on.errors.some((e) => /setState|effect/i.test(e.message))).toBe(true);
  });

  it("leaves unset flags at the compiler's defaults", () => {
    const { code, errors } = transformSync("Component.tsx", fixture, {
      plugins: { reactCompiler: { environment: { enableForest: false } } },
      jsx: { runtime: "automatic" },
    });
    expect(errors).toEqual([]);
    expect(code).toContain("_c(");
  });
});
