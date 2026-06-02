import { defineConfig } from "#oxlint";

const a = defineConfig({}) as any;
const b = defineConfig({ extends: [a] }) as any;
a.extends = [b];

export default defineConfig({
  extends: [a],
});
