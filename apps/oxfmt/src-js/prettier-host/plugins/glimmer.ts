// Stable re-export of `prettier/plugins/glimmer`. See `libs/plugin-resolution.ts`.
// Prettier declares no default in its types, so the runtime default is read off
// the namespace instead of re-exported.
import * as mod from "prettier/plugins/glimmer";

export * from "prettier/plugins/glimmer";

const defaultExport: unknown = (mod as { default?: unknown }).default ?? mod;
export default defaultExport;
