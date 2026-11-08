import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("sortAttributes", () => {
  it.each(["html", "vue", "jsx", "tsx"])("is opt-in for %s", async (extension) => {
    const source =
      extension === "html" || extension === "vue"
        ? '<div z="last" a="first"></div>'
        : 'const element = <div z="last" a="first" />;';
    const filename = `input.${extension}`;
    const defaults = await format(filename, source);
    const disabled = await format(filename, source, { sortAttributes: false });
    const enabled = await format(filename, source, { sortAttributes: true });
    const object = await format(filename, source, { sortAttributes: {} });
    for (const result of [defaults, disabled, enabled, object]) {
      expect(result.errors).toStrictEqual([]);
    }
    expect(disabled.code).toBe(defaults.code);
    expect(enabled.code).toBe(object.code);
    expect(enabled.code).not.toBe(defaults.code);
    expect((await format(filename, enabled.code, { sortAttributes: true })).code).toBe(
      enabled.code,
    );
  });

  it.each([
    { groups: ["["] },
    { groups: ["$MISSING"] },
    { groups: ["$DEFAULT", "$DEFAULT"] },
    { order: "invalid" },
    { group: ["^id$"] },
  ])("rejects invalid configuration %j", async (sortAttributes) => {
    const source = '<div z="last" a="first"></div>';
    // @ts-expect-error: Invalid options must be rejected without changing the input.
    const result = await format("input.html", source, { sortAttributes });
    expect(result.code).toBe(source);
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0].message).toMatch(/sortAttributes/i);
  });

  it("does not reverse equal JSX attributes in descending order", async () => {
    const source = 'const element = <div a="first" a="second" z="last" />;';
    const result = await format("input.jsx", source, { sortAttributes: { order: "desc" } });
    expect(result.errors).toStrictEqual([]);
    expect(result.code.indexOf('a="first"')).toBeLessThan(result.code.indexOf('a="second"'));
    expect(
      (await format("input.jsx", result.code, { sortAttributes: { order: "desc" } })).code,
    ).toBe(result.code);
  });

  it.each([
    ["input.js", 'const element = html`<div z="before" ...${props} a="after"></div>`;'],
    ["input.js", 'const element = html`<div a="before" ...${props} z="after"></div>`;'],
    ["input.js", 'const element = html`<div z="before" ${name}="value" a="after"></div>`;'],
    ["input.js", 'const element = html`<div z="before" data-${key}="value" a="after"></div>`;'],
    ["input.vue", '<template><Widget fooBar="first" foo-bar="second" /></template>'],
    ["input.vue", '<template><Widget foo-bar="first" fooBar="second" /></template>'],
    ["input.vue", '<template><Widget fooBarBaz="first" foo-bar-baz="second" /></template>'],
    ["input.vue", '<template><Widget foo1="first" foo-1="second" /></template>'],
    ["input.vue", '<template><Widget foo_bar="first" foo-_bar="second" /></template>'],
    ["input.tsx", "const element = <C z={z} a={a} />;"],
    ["input.tsx", "const element = <C a={a} z={z} />;"],
  ])("preserves precedence and evaluation order in %s: %s", async (filename, input) => {
    const baseline = await format(filename, input, { sortAttributes: false });
    expect(baseline.errors).toStrictEqual([]);
    const results = await Promise.all(
      [true, { order: "desc" as const }, { groups: ["^a$", "^foo-bar$", "$DEFAULT", "^z$"] }].map(
        (sortAttributes) => format(filename, input, { sortAttributes }),
      ),
    );
    for (const result of results) {
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toBe(baseline.code);
    }
  });

  it.each([
    '<Widget :z="next()" :a="next()" />',
    '<Widget :z="state.z" :a="state.a" />',
    '<Widget :z="z" :a="a" />',
    '<Widget :z="next()" :a />',
    '<Widget v-bind:z="next()" v-bind:a="next()" />',
    '<Widget v-model="one" :modelValue="two" />',
    '<Widget :modelValue="two" v-model="one" />',
    '<Widget v-model:title="one" :title="two" />',
    '<Widget v-on:click="z" @click="a" />',
    '<Widget @click="z" v-on:click="a" />',
    '<Widget @click="z" :onClick="a" />',
    "<div v-z v-a />",
    '<div v-z="next()" v-a="next()" />',
    '<div v-z v-show="visible" v-a />',
  ])("preserves Vue evaluation and directive order in %s", async (template) => {
    const input = `<template>${template}</template>`;
    const baseline = await format("input.vue", input, { sortAttributes: false });
    expect(baseline.errors).toStrictEqual([]);
    const results = await Promise.all(
      [true, { order: "desc" as const }, { groups: ["^v-", "^@", "^:", "$DEFAULT"] }].map(
        (sortAttributes) => format("input.vue", input, { sortAttributes }),
      ),
    );
    for (const result of results) {
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toBe(baseline.code);
    }
  });

  it.each(["html", "vue", "tsx"])("composes with Tailwind sorting in %s", async (extension) => {
    const source =
      extension === "tsx"
        ? 'const element = <div z="last" className="p-4 flex" a="first" />;'
        : '<div z="last" class="p-4 flex" a="first"></div>';
    const options = { sortAttributes: true, sortTailwindcss: true } as const;
    const result = await format(`input.${extension}`, source, options);
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toContain('="flex p-4"');
    expect(result.code.indexOf('a="first"')).toBeLessThan(result.code.indexOf('z="last"'));
    expect((await format(`input.${extension}`, result.code, options)).code).toBe(result.code);
  });
});
