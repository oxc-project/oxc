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

  it("aborts the transform when React Compiler reports an error", () => {
    const { code, errors } = transformSync(
      "Component.jsx",
      `
function Component(props) {
  if (props.cond) {
    useState(0);
  }
  return <div>{props.text}</div>;
}
`,
      {
        plugins: { reactCompiler: true },
        jsx: { runtime: "automatic" },
      },
    );

    // A React Compiler error (Rules of Hooks violation) is fatal: it is surfaced
    // at error severity and the transform stops, emitting no code.
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
