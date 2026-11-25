import { describe, expect, it } from "vitest";

import { isolatedDeclaration, isolatedDeclarationSync } from "../index";

describe("isolated declaration", () => {
  const code = `
  /**
   * jsdoc 1
   */
  export class A {
    /**
     * jsdoc 2
     */
    foo = "bar";
  }
  // Do not keep normal comments
  export class B {}
  `;

  it("matches output", () => {
    const ret = isolatedDeclarationSync("test.ts", code, { sourcemap: true });
    expect(ret).toMatchObject({
      code:
        "/**\n"
        + "* jsdoc 1\n"
        + "*/\n"
        + "export declare class A {\n"
        + "\t/**\n"
        + "\t* jsdoc 2\n"
        + "\t*/\n"
        + "\tfoo: string;\n"
        + "}\n"
        + "export declare class B {}\n",
      map: {
        names: [],
        sources: ["test.ts"],
        sourcesContent: [code],
        version: 3,
      },
      errors: [],
    });
  });

  it("produces same result as sync", async () => {
    const syncResult = isolatedDeclarationSync("test.ts", code, { sourcemap: true });
    const asyncResult = await isolatedDeclaration("test.ts", code, { sourcemap: true });

    expect(asyncResult.code).toEqual(syncResult.code);
    expect(asyncResult.errors.length).toBe(syncResult.errors.length);
    expect(asyncResult.map).toMatchObject(syncResult.map!);
  });

  it.each([isolatedDeclarationSync, isolatedDeclaration])(
    "retains types referenced by exported ambient variables",
    async (transform) => {
      const result = await transform(
        "input.ts",
        `import type { Opts } from "./opts";
interface Internal { i: 1 }
export declare const defaults: Opts;
export declare let state: Internal;`,
      );

      expect(result.errors).toEqual([]);
      expect(result.code).toBe(
        'import type { Opts } from "./opts";\n'
          + "interface Internal {\n\ti: 1;\n}\n"
          + "export declare const defaults: Opts;\n"
          + "export declare let state: Internal;\n"
          + "export {};\n",
      );
    },
  );

  it("evaluates enum template literal initializers", () => {
    const source =
      'export enum Ev { Prefix = "app", Click = `${Prefix}:click`, Multi = `line1\\nline2`, Num = `x${1}y` }';
    const result = isolatedDeclarationSync("test.ts", source);

    expect(result.errors).toEqual([]);
    expect(result.code).toBe(
      "export declare enum Ev {\n"
        + '\tPrefix = "app",\n'
        + '\tClick = "app:click",\n'
        + '\tMulti = "line1\\nline2",\n'
        + '\tNum = "x1y"\n'
        + "}\n",
    );
  });
});
