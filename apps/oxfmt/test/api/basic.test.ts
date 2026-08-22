import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("Format non-js", () => {
  it("should format json with options", async () => {
    const jsoncCode = `
{
  // Package name
  "foo": "my",
  // Trailing comma test
  "bar": "1",
}
`.trim();
    const result = await format("foo.jsonc", jsoncCode, {
      insertFinalNewline: false,
    });
    expect(result.code).toBe(`${jsoncCode}`);
    expect(result.errors).toStrictEqual([]);
  });

  it("should format vue with options", async () => {
    const vueCode = `
<template><div>Vue</div></template>
<style>div{color:red;}</style>
`.trim();
    const result = await format("Component.vue", vueCode, {
      vueIndentScriptAndStyle: true,
    });
    expect(result.code).toBe(
      `
<template><div>Vue</div></template>
<style>
  div {
    color: red;
  }
</style>
`.trimStart(),
    );
    expect(result.errors).toStrictEqual([]);
  });

  it("should format angular component templates ending with .component.html", async () => {
    const angularCode = `@if (  condition  ) {
  <div *ngIf="  isOpen  ">{{   message   }}</div>
}`;
    const result = await format("my.component.html", angularCode, {});
    expect(result.code).toBe(
      `@if (condition) {
  <div *ngIf="isOpen">{{ message }}</div>
}
`,
    );
    expect(result.errors).toStrictEqual([]);
  });

  it("should format .html with sibling .ts as angular template", async () => {
    const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "oxfmt-angular-test-"));
    const tsFile = path.join(tempDir, "mycomponent.ts");
    const htmlFile = path.join(tempDir, "mycomponent.html");
    const standaloneHtmlFile = path.join(tempDir, "standalone.html");

    fs.writeFileSync(tsFile, "export class MyComponent {}");
    fs.writeFileSync(htmlFile, "");
    fs.writeFileSync(standaloneHtmlFile, "");

    const angularCode = `@if (  condition  ) {
  <div *ngIf="  isOpen  ">{{   message   }}</div>
}`;

    try {
      const resultSibling = await format(htmlFile, angularCode, {});
      expect(resultSibling.code).toBe(
        `@if (condition) {
  <div *ngIf="isOpen">{{ message }}</div>
}
`,
      );
      expect(resultSibling.errors).toStrictEqual([]);

      const resultStandalone = await format(standaloneHtmlFile, angularCode, {});
      // Formatted with HTML parser (does not format Angular bindings/control flow expressions)
      expect(resultStandalone.code).toBe(
        `@if ( condition ) {
<div *ngIf="  isOpen  ">{{ message }}</div>
}
`,
      );
      expect(resultStandalone.errors).toStrictEqual([]);
    } finally {
      fs.rmSync(tempDir, { recursive: true, force: true });
    }
  });

  it("should surface Prettier parse errors as-is", async () => {
    const brokenVue = `<template><div></template>`;
    const result = await format("broken.vue", brokenVue, {});

    expect(result.code).toBe(brokenVue);
    expect(result.errors[0]?.message).toMatch(/Unexpected closing tag/);
  });
});

describe("Format empty", () => {
  it("should format empty string", async () => {
    let result = await format("empty.js", "", {});
    expect(result.code).toBe("");
    expect(result.errors).toStrictEqual([]);

    result = await format("empty.toml", "  ", {});
    expect(result.code).toBe("");
    expect(result.errors).toStrictEqual([]);

    result = await format("empty.json", "\n\n", {});
    expect(result.code).toBe("");
    expect(result.errors).toStrictEqual([]);

    result = await format("empty.md", " \n ", {});
    expect(result.code).toBe("");
    expect(result.errors).toStrictEqual([]);
  });
});
