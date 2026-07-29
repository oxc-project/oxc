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

    expect(result.errors).toEqual([]);
    expect(result.code).toContain("react/compiler-runtime");
    expect(result.code).toContain("_c(");
    expect(result.code).not.toContain("interface Props");
    expect(result.code).not.toContain("<button");
    expect(result.code).toContain("@license MIT");
  });

  it("forwards React Compiler options", () => {
    const target = transformSync("Component.tsx", fixture, { target: "18" });
    expect(target.errors).toEqual([]);
    expect(target.code).toContain("react-compiler-runtime");

    const gated = transformSync("Component.tsx", fixture, {
      gating: {
        source: "feature-flags",
        importSpecifierName: "isCompilerEnabled",
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
        dynamicGating: { source: "dynamic-feature-flags" },
      },
    );
    expect(dynamic.errors).toEqual([]);
    expect(dynamic.code).toContain("dynamic-feature-flags");
    expect(dynamic.code).toContain("isCompilerEnabled");

    const meta = transformSync("Component.tsx", fixture, {
      target: {
        kind: "donotuse_meta_internal",
        runtimeModule: "custom-react-runtime",
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
      ignoreUseNoForget: true,
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
    const result = transformSync("Component.tsx", fixture, options as never);
    expect(result.code).toBe("");
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0].message).toContain(`Invalid React Compiler \`${option}\` option`);
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
      sources: ["src/"],
    });

    expect(result.errors).toEqual([]);
    expect(result.code).not.toContain("react/compiler-runtime");
    expect(result.code).not.toContain("interface Props");
    expect(result.code).not.toContain("<button");
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
});

describe("transform", () => {
  it("matches the synchronous transform", async () => {
    const sync = transformSync("Component.tsx", fixture);
    const asyncResult = await transform("Component.tsx", fixture);

    expect(asyncResult).toEqual(sync);
  });
});
