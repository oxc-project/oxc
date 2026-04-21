// @vitest-environment happy-dom
import { describe, expect, test } from "vitest";

import { getModules } from "./utils";

const info = "$type $options";

describe("alpine", async () => {
  const modules = await getModules("alpinejs/dist/", "module.cjs.js", "cjs");
  test.each(modules)(info, ({ module: { Alpine } }) => {
    expect(Alpine.reactive({ count: 1 }).count).toBe(1);
  });
});

describe("lit", async () => {
  const modules = await getModules("lit/all/", "lit-all.min.js", "esm");
  test.each(modules)(info, ({ module: Lit }) => {
    expect(Lit.html`<div>rendered</div>`).toBeDefined();
  });
});

describe("jquery", async () => {
  const modules = await getModules("jquery/dist/", "jquery.js", "cjs");
  test.each(modules)(info, ({ module: jQuery }) => {
    expect(jQuery("<div>rendered</div>").text()).toBe("rendered");
  });
});

describe("d3", async () => {
  const modules = await getModules("d3/dist/", "d3.js", "cjs");
  test.each(modules)(info, ({ module: D3 }) => {
    expect(D3.select("body").append("div").text("rendered").text()).toBe("rendered");
  });
});

describe("motion", async () => {
  const modules = await getModules("motion/dist/", "motion.dev.js", "cjs");
  test.each(modules)(info, ({ module: Motion }) => {
    const element = document.createElement("div");
    expect(() => Motion.animate(element, { rotate: 360, duration: 0.1 })).not.toThrow();
  });
});
