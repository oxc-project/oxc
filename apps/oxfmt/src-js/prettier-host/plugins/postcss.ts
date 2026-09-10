// Stable re-export of `prettier/plugins/postcss`. See `libs/plugin-resolution.ts`.
// Prettier declares no default in its types, so the runtime default is read off
// the namespace instead of re-exported.
import * as mod from "prettier/plugins/postcss";

export * from "prettier/plugins/postcss";

const defaultExport: unknown = (mod as { default?: unknown }).default ?? mod;
export default defaultExport;
